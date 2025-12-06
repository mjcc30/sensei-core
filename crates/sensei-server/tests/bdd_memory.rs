use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use sensei_lib::agents::{Orchestrator, router::RouterAgent};
use sensei_lib::llm::LlmClient;
use sensei_lib::memory::MemoryStore;
use sensei_server::{AppState, app};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn bdd_memory_flow() {
    // 1. Setup
    let memory = MemoryStore::new("sqlite::memory:").await.unwrap();
    memory.migrate().await.unwrap();

    let llm = Arc::new(LlmClient::new("dummy".to_string()));
    let orchestrator = Arc::new(Orchestrator::new());
    let router = Arc::new(RouterAgent::new(llm.clone(), None, "Dummy Prompt"));

    let state = AppState {
        orchestrator,
        router,
        memory: memory.clone(),
        llm,
    };
    let app_router = app(state);

    // 2. First Request
    let req1 = Request::builder()
        .uri("/v1/ask")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "prompt": "My name is Max"
            })
            .to_string(),
        ))
        .unwrap();

    let resp1 = app_router.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    let session_id_header = resp1.headers().get("x-session-id");
    assert!(
        session_id_header.is_some(),
        "Server did not return X-Session-ID"
    );
    let session_id = session_id_header.unwrap().to_str().unwrap().to_string();

    // 3. Verify Persistence
    let session = memory.get_session(&session_id).await;
    assert!(session.is_ok(), "Session was not created in DB");

    // 4. Second Request with Context
    let req2 = Request::builder()
        .uri("/v1/ask")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-session-id", &session_id)
        .body(Body::from(
            json!({
                "prompt": "What is my name?"
            })
            .to_string(),
        ))
        .unwrap();

    let resp2 = app_router.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
}
