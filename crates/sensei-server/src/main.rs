use sensei_server::{app, AppState};
use sensei_server::llm::LlmClient;
use sensei_server::memory::MemoryStore;
use sensei_server::agents::{Orchestrator, router::RouterAgent, specialists::SpecializedAgent};
use sensei_server::config::load_prompts;
use sensei_common::AgentCategory;
use tokio::net::TcpListener;
use std::sync::Arc;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 1. Load Configuration (Prompts)
    // We try to load prompts.yaml from the current directory.
    // In production, this path should be configurable.
    let prompts_path = "prompts.yaml";
    let prompts_config = match load_prompts(prompts_path) {
        Ok(c) => {
            println!("✅ Loaded prompts from {}", prompts_path);
            Some(c)
        },
        Err(e) => {
            eprintln!("⚠️ Failed to load prompts.yaml: {}. Using default prompts.", e);
            None
        }
    };

    // Helper to retrieve prompt or fallback
    let get_prompt = |key: &str, default: &str| -> String {
        if let Some(config) = &prompts_config {
            if let Some(agent_conf) = config.agents.get(key) {
                return agent_conf.prompt.clone();
            }
        }
        default.to_string()
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

    let memory = MemoryStore::new(&db_url).await.expect("Failed to connect to database");
    memory.migrate().await.expect("Failed to migrate database");

    // 4. Init Swarm
    let mut orchestrator = Orchestrator::new();

    // Register Specialists with Dynamic Prompts
    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Red,
        &get_prompt("red_team", "SYSTEM: You are a Red Team Operator. Provide offensive security insights.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Blue,
        &get_prompt("blue_team", "SYSTEM: You are a Blue Team Analyst. Provide defensive security insights.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Cloud,
        &get_prompt("cloud", "SYSTEM: You are a Cloud Security Architect. Audit AWS/Azure/GCP configurations.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Crypto,
        &get_prompt("crypto", "SYSTEM: You are a Cryptographer.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Osint,
        &get_prompt("osint", "SYSTEM: You are an Intelligence Officer.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Casual,
        &get_prompt("casual", "SYSTEM: You are Sensei, a helpful AI assistant.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Novice,
        &get_prompt("novice", "SYSTEM: You are a Teacher.")
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::System,
        &get_prompt("system", "SYSTEM: You are Root.")
    )));

    // 5. Init Router
    // Note: RouterAgent currently has its prompt hardcoded in `router.rs`.
    // Ideally, we should inject it there too.
    let router = Arc::new(RouterAgent::new(llm_client.clone()));

    // 6. Build State
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        router,
        memory
    };

    // 7. Start Server
    let app = app(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Sensei Server running on http://0.0.0.0:3000 (Swarm Mode)");
    axum::serve(listener, app).await.unwrap();
}
