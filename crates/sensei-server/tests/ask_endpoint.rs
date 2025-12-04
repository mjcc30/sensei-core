use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;
use sensei_server::{app, AppState};
use sensei_server::llm::LlmClient;
use serde_json::{json, Value};
use std::sync::Arc;

#[tokio::test]
async fn ask_endpoint_returns_response() {
    let state = AppState {
        llm: Arc::new(LlmClient::new("dummy".to_string()))
    };
    let app = app(state);

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

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // With a dummy key, genai/Gemini will return an error or fail.
    // Our handler catches errors and returns "Error: ...".
    // So content should exist.
    assert!(body.get("content").is_some());

    let content = body["content"].as_str().unwrap();
    println!("Response: {}", content);
}
