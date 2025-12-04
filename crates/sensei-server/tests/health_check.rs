use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sensei_server::llm::LlmClient;
use sensei_server::memory::MemoryStore;
use sensei_server::{AppState, app};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn health_check_works() {
    let memory = MemoryStore::new("sqlite::memory:").await.unwrap();
    // No need to migrate for health check strictly, but good practice

    let state = AppState {
        llm: Arc::new(LlmClient::new("dummy".to_string())),
        memory,
    };
    let app = app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
