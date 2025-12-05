use anyhow::Result;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_server::agents::Agent;
use sensei_server::agents::action::ToolExecutorAgent;
use sensei_server::llm::Llm;
use sensei_server::tools::Tool;
use std::sync::{Arc, Mutex};

struct MockTool {
    was_called: Arc<Mutex<bool>>,
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        "mock_tool"
    }
    async fn execute(&self, _args: &str) -> Result<String> {
        *self.was_called.lock().unwrap() = true;
        Ok("Success".to_string())
    }
}

struct DecisionLlm {
    response: String,
}
#[async_trait]
impl Llm for DecisionLlm {
    async fn generate(&self, _prompt: &str) -> Result<String> {
        Ok(self.response.clone())
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0; 3072])
    }
}

#[tokio::test]
async fn tool_agent_executes_correct_tool() {
    // Setup Shared State
    let tool_called = Arc::new(Mutex::new(false));

    // Setup Agent
    let llm = Arc::new(DecisionLlm {
        response: r#"{"tool_name": "mock_tool", "argument": "run"}"#.to_string(),
    });
    let mut agent = ToolExecutorAgent::new(llm, AgentCategory::Action);

    // Register Tool
    agent.register_tool(Box::new(MockTool {
        was_called: tool_called.clone(),
    }));

    // Act
    let response = agent.process("Please run the mock tool").await;

    // Assert
    assert!(
        *tool_called.lock().unwrap(),
        "Tool should have been executed"
    );
    assert!(
        response.contains("Success"),
        "Response should contain tool output"
    );
}

#[tokio::test]
async fn tool_agent_handles_unknown_tool() {
    let llm = Arc::new(DecisionLlm {
        response: r#"{"tool_name": "unknown_tool", "argument": "run"}"#.to_string(),
    });
    let agent = ToolExecutorAgent::new(llm, AgentCategory::Action);

    let response = agent.process("Run ghost tool").await;

    assert!(response.contains("Tool 'unknown_tool' selected by AI is not found"));
}
