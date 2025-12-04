use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; 
use sensei_server::{app, AppState};
use sensei_server::llm::LlmClient;
use std::sync::Arc;

#[tokio::test]
async fn health_check_works() {
    let state = AppState { 
        llm: Arc::new(LlmClient::new("dummy".to_string())) 
    };
    let app = app(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}