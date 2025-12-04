pub mod llm;
pub mod memory;

use crate::llm::LlmClient;
use crate::memory::MemoryStore;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use sensei_common::{AskRequest, AskResponse, Health};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub llm: Arc<LlmClient>,
    pub memory: MemoryStore,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/ask", post(ask_handler))
        .with_state(state)
}

async fn health_check() -> Json<Health> {
    Json(Health {
        status: "ok".to_string(),
    })
}

async fn ask_handler(
    State(state): State<AppState>,
    Json(payload): Json<AskRequest>,
) -> Json<AskResponse> {
    // 1. Create session or use existing (TODO: accept session_id in request)
    // For now, create a new one for every request (or just log it)

    // Example: Log prompt to DB
    // let _ = state.memory.create_session(Some(&payload.prompt[..10])).await;

    let content = match state.llm.generate(&payload.prompt).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("LLM Error: {}", e);
            "Error: Failed to generate response.".to_string()
        }
    };

    Json(AskResponse { content })
}
