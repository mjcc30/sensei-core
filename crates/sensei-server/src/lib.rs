pub mod agents;
pub mod config;
pub mod llm;
pub mod memory;
pub mod tools;

use crate::agents::Orchestrator;
use crate::agents::router::RouterAgent;
use crate::llm::Llm;
use crate::memory::MemoryStore;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use sensei_common::{AskRequest, AskResponse, Health};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<Orchestrator>,
    pub router: Arc<RouterAgent>,
    pub memory: MemoryStore,
    pub llm: Arc<dyn Llm>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/ask", post(ask_handler))
        .route("/v1/debug/classify", post(debug_classify_handler))
        .route("/v1/knowledge/add", post(add_document_handler))
        .with_state(state)
}

async fn health_check() -> Json<Health> {
    Json(Health {
        status: "ok".to_string(),
    })
}

#[derive(Deserialize)]
struct AddDocumentRequest {
    content: String,
}

async fn add_document_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddDocumentRequest>,
) -> impl IntoResponse {
    // 1. Generate Embedding
    let embedding = match state.llm.embed(&payload.content).await {
        Ok(vec) => vec,
        Err(e) => {
            eprintln!("Embedding Error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to generate embedding"})),
            );
        }
    };

    // 2. Store Document
    if let Err(e) = state.memory.add_document(&payload.content, embedding).await {
        eprintln!("Storage Error: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to store document"})),
        );
    }

    (
        StatusCode::OK,
        Json(json!({"status": "Document ingested successfully"})),
    )
}

async fn debug_classify_handler(
    State(state): State<AppState>,
    Json(payload): Json<AskRequest>,
) -> Json<Value> {
    let decision = state.router.classify(&payload.prompt).await;

    Json(json!({
        "category": decision.category,
        "enhanced_query": decision.query
    }))
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

    // 3.5. RAG Retrieval
    let context_docs = match state.llm.embed(&decision.query).await {
        Ok(embedding) => state
            .memory
            .search_documents(embedding, 3)
            .await
            .unwrap_or_default(),
        Err(e) => {
            eprintln!("RAG Embedding Failed: {}", e);
            vec![]
        }
    };

    let final_prompt = if !context_docs.is_empty() {
        println!("📚 RAG: Found {} relevant documents.", context_docs.len());
        format!(
            "RELEVANT KNOWLEDGE:\n{}\n\nUSER QUERY:\n{}",
            context_docs.join("\n---\n"),
            decision.query
        )
    } else {
        decision.query
    };

    // 4. Dispatch to Agent using context-enriched query
    let content = state
        .orchestrator
        .dispatch(decision.category, &final_prompt)
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
