pub mod llm;

use axum::{
    extract::State,
    routing::{get, post},
    Router,
    Json,
};
use std::sync::Arc;
use crate::llm::LlmClient;
use sensei_common::{Health, AskRequest, AskResponse};

#[derive(Clone)]
pub struct AppState {
    pub llm: Arc<LlmClient>,
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
    Json(payload): Json<AskRequest>
) -> Json<AskResponse> {
    let content = match state.llm.generate(&payload.prompt).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("LLM Error: {}", e);
            "Error: Failed to generate response.".to_string()
        }
    };

    Json(AskResponse {
        content,
    })
}
