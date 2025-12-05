pub mod agents;
pub mod config;
pub mod llm;
pub mod memory;
pub mod tools;

use crate::agents::Orchestrator;
use crate::agents::router::RouterAgent;
use crate::memory::MemoryStore;
use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
};
use sensei_common::{AskRequest, AskResponse, Health};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<Orchestrator>,
    pub router: Arc<RouterAgent>,
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
    // 1. Session ID
    let session_id = if let Some(header_val) = headers.get("x-session-id") {
        header_val.to_str().unwrap_or("").to_string()
    } else {
        state.memory.create_session(None).await.unwrap_or_default()
    };

    if session_id.is_empty() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(AskResponse {
                content: "Failed to init session".to_string(),
            }),
        )
            .into_response();
    }

    // 2. Persist User Message
    if let Err(e) = state
        .memory
        .add_message(&session_id, "user", &payload.prompt)
        .await
    {
        eprintln!("DB Error (User Msg): {}", e);
    }

    // 3. Route Query
    let decision = state.router.classify(&payload.prompt).await;
    println!(
        "🧠 Routing query '{}' to {:?} (Optimized: '{}')",
        payload.prompt, decision.category, decision.query
    );

    // 4. Dispatch to Agent using OPTIMIZED query
    let content = state
        .orchestrator
        .dispatch(decision.category, &decision.query)
        .await;

    // 5. Persist AI Message
    if let Err(e) = state
        .memory
        .add_message(&session_id, "assistant", &content)
        .await
    {
        eprintln!("DB Error (AI Msg): {}", e);
    }

    // 6. Response
    let mut response = Json(AskResponse { content }).into_response();
    if let Ok(header_val) = axum::http::HeaderValue::from_str(&session_id) {
        response.headers_mut().insert("x-session-id", header_val);
    }

    response
}
