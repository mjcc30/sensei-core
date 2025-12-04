use dotenvy::dotenv;
use sensei_server::llm::LlmClient;
use sensei_server::memory::MemoryStore;
use sensei_server::{AppState, app};
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Init LLM
    let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: GEMINI_API_KEY not set.");
        "dummy".to_string()
    });
    let llm_client = Arc::new(LlmClient::new(api_key));

    // Init Memory
    // Use local file or environment variable
    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://sensei.db?mode=rwc".to_string());
    println!("📦 Connecting to database: {}", db_url);

    let memory = MemoryStore::new(&db_url)
        .await
        .expect("Failed to connect to database");
    memory.migrate().await.expect("Failed to migrate database");

    let state = AppState {
        llm: llm_client,
        memory,
    };

    let app = app(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Sensei Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
