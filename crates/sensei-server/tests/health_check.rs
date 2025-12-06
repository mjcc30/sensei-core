use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sensei_lib::agents::{Orchestrator, router::RouterAgent};
use sensei_lib::llm::GeminiClient;
use sensei_lib::memory::MemoryStore;
use sensei_server::{AppState, app};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn health_check_works() {
    let memory = MemoryStore::new("sqlite::memory:").await.unwrap();
    let llm = Arc::new(GeminiClient::new("dummy"));

    let state = AppState {
        orchestrator: Arc::new(Orchestrator::new()),
        router: Arc::new(RouterAgent::new(llm.clone(), None, "Dummy Prompt")),
        memory,
        llm,
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
