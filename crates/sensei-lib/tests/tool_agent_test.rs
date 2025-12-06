use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_lib::agents::Agent;
use sensei_lib::agents::action::ToolExecutorAgent;
use sensei_lib::errors::SenseiError;
use sensei_lib::llm::Llm;
use sensei_lib::tools::Tool;
use std::sync::{Arc, Mutex};

struct MockTool {
    name: String,
    was_called: Arc<Mutex<bool>>,
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }
    async fn execute(&self, _args: &str) -> Result<String, SenseiError> {
        *self.was_called.lock().unwrap() = true;
        Ok("Success".to_string())
    }
}

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
async fn tool_agent_executes_correct_tool() {
    let tool_called = Arc::new(Mutex::new(false));

    let llm = Arc::new(MockLlm {
        response: r#"{"tool_name": "mock_tool", "argument": "run"}"#.to_string(),
    });

    let mut agent = ToolExecutorAgent::new(llm, AgentCategory::new("action"));

    agent.register_tool(Box::new(MockTool {
        name: "mock_tool".to_string(),
        was_called: tool_called.clone(),
    }));

    let response = agent.process("Run mock tool").await;

    assert!(
        *tool_called.lock().unwrap(),
        "Tool should have been executed"
    );
    assert!(response.contains("Success"));
}

#[tokio::test]
async fn tool_agent_handles_unknown_tool() {
    let llm = Arc::new(MockLlm {
        response: r#"{"tool_name": "ghost_tool", "argument": "run"}"#.to_string(),
    });

    let agent = ToolExecutorAgent::new(llm, AgentCategory::new("action"));
    let response = agent.process("Run ghost tool").await;

    assert!(response.contains("not found in registry"));
}
