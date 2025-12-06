use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_lib::agents::Agent;
use sensei_lib::agents::specialists::SpecializedAgent;
use sensei_lib::llm::Llm;
use sensei_lib::errors::SenseiError;
use std::sync::{Arc, Mutex};

struct MockLlm {
    last_prompt: Mutex<String>,
}

#[async_trait]
impl Llm for MockLlm {
    async fn generate(&self, prompt: &str) -> Result<String, SenseiError> {
        *self.last_prompt.lock().unwrap() = prompt.to_string();
        Ok("Response".to_string())
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, SenseiError> {
        Ok(vec![])
    }
    async fn generate_raw(&self, prompt: &str) -> Result<String, SenseiError> {
        self.generate(prompt).await
    }
}

#[tokio::test]
async fn specialist_uses_system_prompt() {
    let llm = Arc::new(MockLlm {
        last_prompt: Mutex::new(String::new()),
    });
    let agent = SpecializedAgent::new(llm.clone(), AgentCategory::Blue, "SYSTEM PROMPT", None);

    agent.process("Query").await;

    let prompt = llm.last_prompt.lock().unwrap().clone();
    assert!(prompt.contains("SYSTEM PROMPT"));
    assert!(prompt.contains("Query"));
}

#[tokio::test]
async fn specialist_uses_master_prompt_in_raw_mode() {
    let llm = Arc::new(MockLlm {
        last_prompt: Mutex::new(String::new()),
    });
    let agent = SpecializedAgent::new(
        llm.clone(), 
        AgentCategory::Red, 
        "SYSTEM PROMPT", 
        Some("MASTER PROMPT".to_string())
    );

    agent.process("Query --raw").await;

    let prompt = llm.last_prompt.lock().unwrap().clone();
    assert!(prompt.contains("MASTER PROMPT"));
    assert!(!prompt.contains("SYSTEM PROMPT"));
}