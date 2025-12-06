use anyhow::Result;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_lib::agents::mcp_agent::McpAgent;
use sensei_lib::agents::{Agent, Orchestrator};
use sensei_lib::errors::SenseiError;
use sensei_lib::llm::Llm;
use sensei_lib::mcp_client::McpClient;
use std::path::PathBuf;
use std::sync::Arc;

struct DelegatorAgent;
#[async_trait]
impl Agent for DelegatorAgent {
    async fn process(&self, input: &str) -> String {
        if input.contains("[OBSERVATION") {
            return format!("Success! {}", input);
        }
        // The magic string that triggers delegation
        "[DELEGATE: loopback] uptime".to_string()
    }
    fn category(&self) -> AgentCategory {
        AgentCategory::Casual
    }
}

struct DecisionLlm;
#[async_trait]
impl Llm for DecisionLlm {
    async fn generate(&self, _prompt: &str) -> Result<String, SenseiError> {
        Ok(r#"{"tool_name": "system_diagnostic", "arguments": {"command": "uptime"}}"#.to_string())
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, SenseiError> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_a2a_loopback_delegation() -> Result<()> {
    let bin_path = PathBuf::from("../../target/release/sensei-mcp");
    if !bin_path.exists() {
        println!("⚠️ Skipping A2A test: sensei-mcp binary not found");
        return Ok(());
    }

    let orchestrator = Orchestrator::new();

    // 1. Register CASUAL Agent (The Delegator)
    orchestrator.register(Box::new(DelegatorAgent)).await;

    // 2. Register LOOPBACK Agent (The MCP Extension)
    // Connecting to actual binary
    let mcp_client = McpClient::new(bin_path.to_str().unwrap(), &[], None).await?;
    let mcp_agent = McpAgent::new(Arc::new(mcp_client), Arc::new(DecisionLlm), "loopback").await?;

    // Registering it. category() is now Extension("loopback")
    orchestrator.register(Box::new(mcp_agent)).await;
    // 3. Dispatch to CASUAL
    // It should delegate to LOOPBACK, which calls `system_diagnostic` -> `uptime`
    let response = orchestrator
        .dispatch(AgentCategory::Casual, "Do the thing")
        .await;

    println!("Orchestrator Final Response:\n{}", response);

    // 4. Validation
    // The response should contain the observation from the delegated agent
    assert!(response.contains("[OBSERVATION from loopback]"));
    assert!(response.contains("Tool 'system_diagnostic' on loopback executed successfully"));
    // Output of uptime contains "load average" or time
    // assert!(response.contains("load average") || response.contains(":"));

    Ok(())
}
