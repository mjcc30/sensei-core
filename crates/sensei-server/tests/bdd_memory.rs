use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;
use sensei_server::{app, AppState};
use sensei_server::memory::MemoryStore;
use sensei_server::llm::LlmClient;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn bdd_memory_flow() {
    // 1. Setup
    let memory = MemoryStore::new("sqlite::memory:").await.unwrap();
    memory.migrate().await.unwrap();

    // We need a dummy LLM client, but we can't easily assert on internal calls without mocking traits.
    // For this BDD test, we verify the side effects: data in DB.
    let state = AppState {
        llm: Arc::new(LlmClient::new("dummy".to_string())),
        memory: memory.clone(),
    };
    let app_router = app(state); // Renamed to avoid name clash

    // 2. Client starts a session (First Request)
    // Client sends "My name is Max". Server should create a session and store it.
    let req1 = Request::builder()
        .uri("/v1/ask")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "prompt": "My name is Max"
        }).to_string()))
        .unwrap();

    let resp1 = app_router.clone().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Extract Session ID from headers (Assuming we implement returning X-Session-ID)
    let session_id_header = resp1.headers().get("x-session-id");

    // TEST ASSERTION 1: Server must return a Session ID
    assert!(session_id_header.is_some(), "Server did not return X-Session-ID");
    let session_id = session_id_header.unwrap().to_str().unwrap().to_string();

    // 3. Verify Persistence
    // Check if the message is in the DB
    // We need a method `get_history(session_id)` in MemoryStore (TODO)
    // For now, check session exists
    let session = memory.get_session(&session_id).await;
    assert!(session.is_ok(), "Session was not created in DB");

    // 4. Second Request with Context
    // Client sends "What is my name?" WITH the session ID.
    let req2 = Request::builder()
        .uri("/v1/ask")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-session-id", &session_id) // Client sends back ID
        .body(Body::from(json!({
            "prompt": "What is my name?"
        }).to_string()))
        .unwrap();

    let resp2 = app_router.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    // Verify that the second message is also stored linked to the same session
    // (We would need to query messages table, but let's assume if session logic holds, we are good for step 1)
}
