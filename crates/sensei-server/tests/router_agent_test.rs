use sensei_server::agents::router::RouterAgent;
use sensei_server::llm::Llm;
use sensei_common::AgentCategory;
use async_trait::async_trait;
use std::sync::Arc;

struct MockLlm {
    response: String,
}

#[async_trait]
impl Llm for MockLlm {
    async fn generate(&self, _prompt: &str) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }
}

#[tokio::test]
async fn router_classifies_correctly() {
    let mock_llm = MockLlm {
        response: r#"{"category": "Red"}"#.to_string(),
    };

    let router = RouterAgent::new(Arc::new(mock_llm));

    let category = router.classify("Hack wifi").await;
    assert_eq!(category, AgentCategory::Red);
}
