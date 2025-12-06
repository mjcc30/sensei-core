use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_lib::agents::router::RouterAgent;
use sensei_lib::errors::SenseiError;
use sensei_lib::llm::Llm;
use std::sync::Arc;

struct MockLlm {
    response: String,
}

#[async_trait]
impl Llm for MockLlm {
    async fn generate(&self, _prompt: &str) -> Result<String, SenseiError> {
        Ok(self.response.clone())
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, SenseiError> {
        Ok(vec![])
    }
    async fn generate_raw(&self, prompt: &str) -> Result<String, SenseiError> {
        self.generate(prompt).await
    }
}

#[tokio::test]
async fn router_classifies_correctly() {
    let mock = MockLlm {
        response: r#"{"category": "RED", "enhanced_query": "Attack"}"#.to_string(),
    };
    let router = RouterAgent::new(Arc::new(mock), None, "System Prompt");

    let decision = router.classify("hack").await;

    assert_eq!(decision.category, AgentCategory::Red);
    assert_eq!(decision.query, "Attack");
}

#[tokio::test]
async fn router_handles_json_errors() {
    let mock = MockLlm {
        response: "Not JSON".to_string(),
    };
    let router = RouterAgent::new(Arc::new(mock), None, "System Prompt");

    let decision = router.classify("hack").await;

    assert_eq!(decision.category, AgentCategory::Unknown);
    assert_eq!(decision.query, "hack");
}
