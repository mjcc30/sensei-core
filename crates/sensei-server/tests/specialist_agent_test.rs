use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_server::agents::Agent;
use sensei_server::agents::specialists::SpecializedAgent;
use sensei_server::llm::Llm;
use std::sync::{Arc, Mutex};

struct SpyLlm {
    last_prompt: Mutex<String>,
}

impl SpyLlm {
    fn new() -> Self {
        Self {
            last_prompt: Mutex::new(String::new()),
        }
    }
}

#[async_trait]
impl Llm for SpyLlm {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        *self.last_prompt.lock().unwrap() = prompt.to_string();
        Ok("Mock Response".to_string())
    }
}

#[tokio::test]
async fn specialist_injects_system_prompt() {
    let spy = Arc::new(SpyLlm::new());

    let agent = SpecializedAgent::new(spy.clone(), AgentCategory::Red, "YOU ARE RED TEAM.");

    let _response = agent.process("How to hack?").await;

    let sent_prompt = spy.last_prompt.lock().unwrap().clone();

    assert!(
        sent_prompt.contains("YOU ARE RED TEAM."),
        "System prompt missing"
    );
    assert!(sent_prompt.contains("How to hack?"), "User input missing");
}
