use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use sensei_server::agents::{Orchestrator, router::RouterAgent};
use sensei_server::llm::LlmClient;
use sensei_server::memory::MemoryStore;
use sensei_server::{AppState, app};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn ask_endpoint_returns_response() {
    let memory = MemoryStore::new("sqlite::memory:").await.unwrap();
    memory.migrate().await.unwrap();

    let llm = Arc::new(LlmClient::new("dummy".to_string()));
    let orchestrator = Arc::new(Orchestrator::new());
    let router = Arc::new(RouterAgent::new(llm.clone(), "Dummy Prompt"));

        let state = AppState {
            orchestrator,
            router,
            memory,
            llm,
        };    let app = app(state);

    let request_body = json!({
        "prompt": "Hello Sensei"
    });

    let request = Request::builder()
        .uri("/v1/ask")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Response content might be "No agent found" because Orchestrator is empty
    // But it proves the pipeline works.
    assert!(body.get("content").is_some());
}
