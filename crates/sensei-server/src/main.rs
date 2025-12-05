use anyhow::Context;
use dotenvy::dotenv;
use sensei_common::AgentCategory;
use sensei_server::agents::{
    Orchestrator, action::ToolExecutorAgent, router::RouterAgent, specialists::SpecializedAgent,
};
use sensei_server::config::load_prompts;
use sensei_server::llm::{LlmClient, MODEL_CHAT_FAST, MODEL_CHAT_SMART};
use sensei_server::memory::MemoryStore;
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
    let prompts_path = "prompts.yaml";
    let prompts_config = match load_prompts(prompts_path) {
        Ok(c) => {
            info!("✅ Loaded prompts from {}", prompts_path);
            Some(c)
        }
        Err(e) => {
            warn!(
                "⚠️ Failed to load prompts.yaml: {}. Using default prompts.",
                e
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
    action_agent.register_tool(Box::new(sensei_server::tools::nmap::NmapTool));
    orchestrator.register(Box::new(action_agent));

    let mut system_agent = ToolExecutorAgent::new(fast_llm.clone(), AgentCategory::System);
    system_agent.register_tool(Box::new(sensei_server::tools::system::SystemTool));
    orchestrator.register(Box::new(system_agent));

    // 5. Init Router -> Fast LLM
    let router_prompt = get_prompt(
        "router",
        r#"
        You are a Query Optimizer.
        Classify user input into: Red, Blue, Osint, Cloud, Crypto, System, Action, Casual, Novice.
        Output strictly JSON format: {"category": "CategoryName", "enhanced_query": "Query"}
        "#,
    );
    let router = Arc::new(RouterAgent::new(fast_llm.clone(), &router_prompt));

    // 6. Build State
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        router,
        memory,
        llm: smart_llm.clone(),
    };

    // 7. Start Server
    let app = app(state);
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    info!("🚀 Sensei Server running on http://{} (Swarm Mode)", addr);
    info!("⚡ Fast Model: {}", MODEL_CHAT_FAST);
    info!("🧠 Smart Model: {}", MODEL_CHAT_SMART);

    axum::serve(listener, app).await.context("Server crashed")?;

    Ok(())
}
