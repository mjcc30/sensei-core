use dotenvy::dotenv;
use sensei_common::AgentCategory;
use sensei_server::agents::{
    Orchestrator, action::ToolExecutorAgent, router::RouterAgent, specialists::SpecializedAgent,
};
use sensei_server::config::load_prompts;
use sensei_server::llm::LlmClient;
use sensei_server::memory::MemoryStore;
use sensei_server::{AppState, app};
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 1. Load Configuration (Prompts)
    let prompts_path = "prompts.yaml";
    let prompts_config = match load_prompts(prompts_path) {
        Ok(c) => {
            println!("✅ Loaded prompts from {}", prompts_path);
            Some(c)
        }
        Err(e) => {
            eprintln!(
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

    // 2. Init LLM
    let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: GEMINI_API_KEY not set.");
        "dummy".to_string()
    });
    let llm_client = Arc::new(LlmClient::new(api_key));

    // 3. Init Memory
    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://sensei.db?mode=rwc".to_string());
    println!("📦 Connecting to database: {}", db_url);

    let memory = MemoryStore::new(&db_url)
        .await
        .expect("Failed to connect to database");
    memory.migrate().await.expect("Failed to migrate database");

    // 4. Init Swarm
    let mut orchestrator = Orchestrator::new();

    // Register Specialists
    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Red,
        &get_prompt("red_team", "SYSTEM: You are a Red Team Operator."),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Blue,
        &get_prompt("blue_team", "SYSTEM: You are a Blue Team Analyst."),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Cloud,
        &get_prompt("cloud", "SYSTEM: You are a Cloud Security Architect."),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Crypto,
        &get_prompt("crypto", "SYSTEM: You are a Cryptographer."),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Osint,
        &get_prompt("osint", "SYSTEM: You are an Intelligence Officer."),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Casual,
        &get_prompt("casual", "SYSTEM: You are Sensei."),
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Novice,
        &get_prompt("novice", "SYSTEM: You are a Teacher."),
    )));

    // Register Tool Agents (Action & System)
    let mut action_agent = ToolExecutorAgent::new(llm_client.clone(), AgentCategory::Action);
    action_agent.register_tool(Box::new(sensei_server::tools::nmap::NmapTool));
    orchestrator.register(Box::new(action_agent));

    let mut system_agent = ToolExecutorAgent::new(llm_client.clone(), AgentCategory::System);
    system_agent.register_tool(Box::new(sensei_server::tools::system::SystemTool));
    orchestrator.register(Box::new(system_agent));

    // 5. Init Router
    let router_prompt = get_prompt(
        "router",
        r#"
        You are a Query Optimizer.
        Classify user input into: Red, Blue, Osint, Cloud, Crypto, System, Action, Casual, Novice.
        Output strictly JSON format: {"category": "CategoryName", "enhanced_query": "Query"}
        "#,
    );
    let router = Arc::new(RouterAgent::new(llm_client.clone(), &router_prompt));

    // 6. Build State
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        router,
        memory,
        llm: llm_client,
    };

    // 7. Start Server
    let app = app(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Sensei Server running on http://0.0.0.0:3000 (Swarm Mode)");
    axum::serve(listener, app).await.unwrap();
}
