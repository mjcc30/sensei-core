pub mod llm;
pub mod memory;

use axum::{
    extract::State,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
    Router,
    Json,
};
use std::sync::Arc;
use crate::llm::LlmClient;
use crate::memory::MemoryStore;
use sensei_common::{Health, AskRequest, AskResponse};

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
    headers: HeaderMap,
    Json(payload): Json<AskRequest>,
) -> impl IntoResponse {
    // 1. Resolve Session ID
    let session_id = if let Some(header_val) = headers.get("x-session-id") {
        // Check if session exists ? Ideally yes. If not found, create new ?
        // For simple robustnes: use it. If add_message fails due to FK, handle it.
        // Assuming client sends valid ID if it sends one.
        header_val.to_str().unwrap_or("").to_string()
    } else {
        // Create new session
        state.memory.create_session(None).await.unwrap_or_default()
    };

    if session_id.is_empty() {
         return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(AskResponse { content: "Failed to init session".to_string() })
        ).into_response();
    }

    // 2. Persist User Message
    if let Err(e) = state.memory.add_message(&session_id, "user", &payload.prompt).await {
        eprintln!("DB Error (User Msg): {}", e);
        // Maybe session ID was invalid? Recovery logic could be here.
    }

    // 3. Generate (TODO: Inject history)
    let content = match state.llm.generate(&payload.prompt).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("LLM Error: {}", e);
            "Error: Failed to generate response.".to_string()
        }
    };

    // 4. Persist AI Message
    if let Err(e) = state.memory.add_message(&session_id, "assistant", &content).await {
        eprintln!("DB Error (AI Msg): {}", e);
    }

    // 5. Build Response with Header
    let mut response = Json(AskResponse { content }).into_response();

    // x-session-id is typically ASCII/UUID, safe to unwrap
    if let Ok(header_val) = axum::http::HeaderValue::from_str(&session_id) {
        response.headers_mut().insert("x-session-id", header_val);
    }

    response
}
