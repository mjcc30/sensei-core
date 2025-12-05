use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_server::agents::router::RouterAgent;
use sensei_server::llm::Llm;
use std::sync::Arc;

struct MockLlm {
    response: String,
}

#[async_trait]
impl Llm for MockLlm {
    async fn generate(&self, _prompt: &str) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }

    async fn embed(&self, _text: &str) -> anyhow::Result<Vec<f32>> {
        Ok(vec![0.0; 3072])
    }

    async fn generate_raw(&self, prompt: &str) -> anyhow::Result<String> {
        self.generate(prompt).await
    }
}

#[tokio::test]
async fn router_classifies_and_optimizes() {
    let mock_llm = MockLlm {
        response: r#"{"category": "RED", "enhanced_query": "Optimized Attack"}"#.to_string(),
    };

    let router = RouterAgent::new(Arc::new(mock_llm), "Dummy Prompt");
    let decision = router.classify("Hack wifi").await;
    assert_eq!(decision.category, AgentCategory::Red);
    assert_eq!(decision.query, "Optimized Attack");
}
