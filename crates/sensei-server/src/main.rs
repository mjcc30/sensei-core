use dotenvy::dotenv;
use sensei_common::AgentCategory;
use sensei_server::agents::{Orchestrator, router::RouterAgent, specialists::SpecializedAgent};
use sensei_server::llm::LlmClient;
use sensei_server::memory::MemoryStore;
use sensei_server::{AppState, app};
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 1. Init LLM
    let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: GEMINI_API_KEY not set.");
        "dummy".to_string()
    });
    // LlmClient implements Llm trait
    let llm_client = Arc::new(LlmClient::new(api_key));

    // 2. Init Memory
    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://sensei.db?mode=rwc".to_string());
    println!("📦 Connecting to database: {}", db_url);

    let memory = MemoryStore::new(&db_url)
        .await
        .expect("Failed to connect to database");
    memory.migrate().await.expect("Failed to migrate database");

    // 3. Init Swarm
    let mut orchestrator = Orchestrator::new();

    // Register Agents
    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Red,
        "SYSTEM: You are a Red Team Operator. Provide offensive security insights.",
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Blue,
        "SYSTEM: You are a Blue Team Analyst. Provide defensive security insights.",
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Cloud,
        "SYSTEM: You are a Cloud Security Architect. Audit AWS/Azure/GCP configurations.",
    )));

    orchestrator.register(Box::new(SpecializedAgent::new(
        llm_client.clone(),
        AgentCategory::Casual,
        "SYSTEM: You are Sensei, a helpful AI assistant.",
    )));

    // 4. Init Router
    let router = Arc::new(RouterAgent::new(llm_client.clone()));

    // 5. Build State
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        router,
        memory,
    };

    // 6. Start Server
    let app = app(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Sensei Server running on http://0.0.0.0:3000 (Swarm Mode)");
    axum::serve(listener, app).await.unwrap();
}
