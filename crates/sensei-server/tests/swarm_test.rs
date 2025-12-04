use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_server::agents::{Agent, Orchestrator};

struct MockRedAgent;

#[async_trait]
impl Agent for MockRedAgent {
    async fn process(&self, _input: &str) -> String {
        "Red Team Attack Plan".to_string()
    }
    fn category(&self) -> AgentCategory {
        AgentCategory::Red
    }
}

#[tokio::test]
async fn swarm_routing_works() {
    let mut orch = Orchestrator::new();

    // Register the mock agent
    orch.register(Box::new(MockRedAgent));

    // Dispatch directly to RED category (bypassing LLM Router for this unit test)
    let response = orch.dispatch(AgentCategory::Red, "Hack wifi").await;

    assert_eq!(response, "Red Team Attack Plan");

    // Test fallback (unknown category -> Casual or Error?)
    // Assuming default behavior for now returns empty or error string
    let response_blue = orch.dispatch(AgentCategory::Blue, "Analyze logs").await;
    assert!(response_blue.contains("No agent") || response_blue.is_empty());
}
