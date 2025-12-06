use anyhow::Context;
use dotenvy::dotenv;
use sensei_common::AgentCategory;
use sensei_lib::agents::{
    Orchestrator, action::ToolExecutorAgent, router::RouterAgent, specialists::SpecializedAgent,
};
use sensei_lib::config::load_prompts;
use sensei_lib::llm::{LlmClient, MODEL_CHAT_FAST, MODEL_CHAT_SMART};
use sensei_lib::memory::MemoryStore;
use sensei_server::{AppState, app};
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // 0. Init Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    // 1. Load Configuration (Prompts)
    let prompts_path =
        env::var("SENSEI_PROMPTS_PATH").unwrap_or_else(|_| "prompts.yaml".to_string());
    let prompts_config = match load_prompts(&prompts_path) {
        Ok(c) => {
            info!("✅ Loaded prompts from {}", prompts_path);
            Some(c)
        }
        Err(e) => {
            warn!(
                "⚠️ Failed to load prompts from '{}': {}. Using default prompts.",
                prompts_path, e
            );
            None
        }
    };

    let get_prompt = |key: &str, default: &str| -> String {
        prompts_config
            .as_ref()
            .and_then(|config| config.agents.get(key))
            .map(|agent_conf| agent_conf.prompt.clone())
            .unwrap_or_else(|| default.to_string())
    };

    // 2. Init LLM Clients
    let api_key = env::var("GEMINI_API_KEY").context("GEMINI_API_KEY must be set")?;

    // ⚡ Fast Tier
    let fast_llm = Arc::new(LlmClient::new_with_model(api_key.clone(), MODEL_CHAT_FAST));

    // 🧠 Smart Tier
    let smart_llm = Arc::new(LlmClient::new_with_model(api_key.clone(), MODEL_CHAT_SMART));

    // 3. Init Memory
    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://sensei.db?mode=rwc".to_string());
    info!("📦 Connecting to database: {}", db_url);

    let memory = MemoryStore::new(&db_url)
        .await
        .context("Failed to connect to database")?;

    memory
        .migrate()
        .await
        .context("Failed to migrate database")?;

    // 4. Init Swarm
    let mut orchestrator = Orchestrator::new();
    let _active_categories = vec![
        "RED", "BLUE", "CLOUD", "CRYPTO", "OSINT", "SYSTEM", "ACTION", "CASUAL", "NOVICE",
    ];

    // Specialists -> Smart LLM
    orchestrator.register(Box::new(SpecializedAgent::new(
        smart_llm.clone(),
        AgentCategory::Red,
        &get_prompt("red_team", "SYSTEM: You are a Red Team Operator."),
        Some(get_prompt("master", "SYSTEM: You are SENSEI.")),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        smart_llm.clone(),
        AgentCategory::Blue,
        &get_prompt("blue_team", "SYSTEM: You are a Blue Team Analyst."),
        None,
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        smart_llm.clone(),
        AgentCategory::Cloud,
        &get_prompt("cloud", "SYSTEM: You are a Cloud Security Architect."),
        None,
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        smart_llm.clone(),
        AgentCategory::Crypto,
        &get_prompt("crypto", "SYSTEM: You are a Cryptographer."),
        None,
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        smart_llm.clone(),
        AgentCategory::Osint,
        &get_prompt("osint", "SYSTEM: You are an Intelligence Officer."),
        None,
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        smart_llm.clone(),
        AgentCategory::System,
        &get_prompt("system", "SYSTEM: You are Root."),
        None,
    )));

    // Casual/Novice -> Fast LLM
    orchestrator.register(Box::new(SpecializedAgent::new(
        fast_llm.clone(),
        AgentCategory::Casual,
        &get_prompt("casual", "SYSTEM: You are Sensei."),
        None,
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        fast_llm.clone(),
        AgentCategory::Novice,
        &get_prompt("novice", "SYSTEM: You are a Teacher."),
        None,
    )));

    // Register Tool Agents (Action & System Tools)
    let mut action_agent = ToolExecutorAgent::new(fast_llm.clone(), AgentCategory::Action);
    action_agent.register_tool(Box::new(sensei_lib::tools::nmap::NmapTool));
    orchestrator.register(Box::new(action_agent));

    let mut system_agent = ToolExecutorAgent::new(fast_llm.clone(), AgentCategory::System);
    system_agent.register_tool(Box::new(sensei_lib::tools::system::SystemTool));
    orchestrator.register(Box::new(system_agent));

    // 4.5 Init MCP Agents (Dynamic)
    let mut dynamic_extensions = Vec::new();
    let mcp_path = env::var("SENSEI_MCP_CONFIG").unwrap_or("mcp_settings.json".to_string());

    if let Ok(mcp_config) = sensei_lib::config::load_mcp_settings(&mcp_path) {
        info!("🔌 Loading MCP Servers from {}", mcp_path);
        for (name, conf) in mcp_config.mcp_servers {
            let envs = conf.env;
            let args_str: Vec<&str> = conf.args.iter().map(|s| s.as_str()).collect();

            info!("   - Connecting to {}...", name);
            match sensei_lib::mcp_client::McpClient::new(&conf.command, &args_str, envs).await {
                Ok(client) => {
                    let client_arc = Arc::new(client);
                    match sensei_lib::agents::mcp_agent::McpAgent::new(
                        client_arc,
                        fast_llm.clone(),
                        &name,
                    )
                    .await
                    {
                        Ok(agent) => {
                            info!("   ✅ MCP Agent '{}' registered", name);
                            orchestrator.register(Box::new(agent));
                            dynamic_extensions.push(name.to_uppercase());
                        }
                        Err(e) => warn!("   ❌ Failed to init MCP Agent '{}': {}", name, e),
                    }
                }
                Err(e) => warn!("   ❌ Failed to spawn MCP Server '{}': {}", name, e),
            }
        }
    }

    // 5. Init Router -> Fast LLM
    let mut router_prompt = get_prompt(
        "router",
        r#"
        You are a Query Optimizer.
        STANDARD CATEGORIES: RED, BLUE, OSINT, CLOUD, CRYPTO, SYSTEM, ACTION, CASUAL, NOVICE.
        ACTIVE EXTENSIONS: {EXTENSIONS}
        Classify user input into one of the above categories.
        Output strictly JSON format: {"category": "CategoryName", "enhanced_query": "Query"}
        "#,
    );

    let extensions_str = if dynamic_extensions.is_empty() {
        "NONE".to_string()
    } else {
        dynamic_extensions.join(", ")
    };

    router_prompt = router_prompt.replace("{EXTENSIONS}", &extensions_str);
    info!(
        "🧠 Router Prompt Configured with Extensions: {}",
        extensions_str
    );

    let router = Arc::new(RouterAgent::new(
        fast_llm.clone(),
        Some(memory.clone()),
        &router_prompt,
    ));

    // 6. Build State
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        router,
        memory,
        llm: smart_llm.clone(),
    };

    // 7. Start Server
    let app = app(state);
    let listen_target = env::var("SENSEI_LISTEN_ADDR").unwrap_or("0.0.0.0:3000".to_string());

    if listen_target.starts_with("unix://") {
        #[cfg(unix)]
        {
            let path = listen_target.trim_start_matches("unix://");
            if std::fs::metadata(path).is_ok() {
                std::fs::remove_file(path).context("Failed to remove existing socket file")?;
            }

            let listener =
                tokio::net::UnixListener::bind(path).context("Failed to bind to Unix socket")?;

            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;

            info!("🚀 Sensei Server listening on Unix Socket: {}", path);
            axum::serve(listener, app).await.context("Server crashed")?;
        }
        #[cfg(not(unix))]
        {
            anyhow::bail!("Unix Domain Sockets are not supported on this OS");
        }
    } else {
        let listener = TcpListener::bind(&listen_target)
            .await
            .context(format!("Failed to bind to {}", listen_target))?;

        info!(
            "🚀 Sensei Server running on http://{} (Swarm Mode)",
            listen_target
        );
        info!("⚡ Fast Model: {}", MODEL_CHAT_FAST);
        info!("🧠 Smart Model: {}", MODEL_CHAT_SMART);

        axum::serve(listener, app).await.context("Server crashed")?;
    }

    Ok(())
}
