use sensei_server::{app, AppState};
use sensei_server::llm::LlmClient; // Need to make llm module public or re-export LlmClient
use tokio::net::TcpListener;
use std::sync::Arc;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    // Init LLM Client
    // In production, we fail fast if key is missing
    let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: GEMINI_API_KEY not set. LLM calls will fail.");
        "dummy".to_string()
    });
    
    let llm_client = Arc::new(LlmClient::new(api_key));
    let state = AppState { llm: llm_client };

    let app = app(state);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    println!("🚀 Sensei Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
