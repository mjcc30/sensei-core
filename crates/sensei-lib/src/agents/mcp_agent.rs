use crate::agents::Agent;
use crate::llm::Llm;
use crate::mcp_client::McpClient;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

pub struct McpAgent {
    client: Arc<McpClient>,
    llm: Arc<dyn Llm>,
    server_name: String,
    tools: Vec<Value>, // Cache of available tools descriptions
}

#[derive(Deserialize)]
struct McpToolCall {
    tool_name: String,
    arguments: Value,
}

impl McpAgent {
    pub async fn new(
        client: Arc<McpClient>,
        llm: Arc<dyn Llm>,
        server_name: &str,
    ) -> anyhow::Result<Self> {
        // Auto-discovery of tools
        client.initialize().await?;
        let tools = client.list_tools().await?;

        println!(
            "🔌 Connected to MCP Server '{}'. Discovered {} tools.",
            server_name,
            tools.len()
        );

        Ok(Self {
            client,
            llm,
            server_name: server_name.to_string(),
            tools,
        })
    }

    async fn decide_tool(&self, query: &str) -> Option<McpToolCall> {
        // Construct a prompt that describes available MCP tools
        let tools_desc = serde_json::to_string_pretty(&self.tools).unwrap_or_default();

        let prompt = format!(
            r#" 
            You are an autonomous Agent controlling an MCP Server named '{}'.
            
            Available Tools (JSON Schema):
            {} 
            
            Task: Analyze the user request and decide which tool to execute.
            User Request: "{}" 
            
            Rules:
            - If the request matches a tool, output strictly JSON: {{ "tool_name": "name", "arguments": {{ ... }} }}
            - If NO tool matches, output strictly JSON: {{ "tool_name": "none", "arguments": {{}} }} 
            
            Output strictly JSON.
            "#,
            self.server_name, tools_desc, query
        );

        let response = self.llm.generate(&prompt).await.ok()?;

        let start = response.find('{').unwrap_or(0);
        let end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[start..end];

        serde_json::from_str::<McpToolCall>(json_str).ok()
    }
}

#[async_trait]
impl Agent for McpAgent {
    async fn process(&self, input: &str) -> String {
        // 1. LLM decides which tool to call
        if let Some(call) = self.decide_tool(input).await {
            if call.tool_name == "none" {
                return format!(
                    "I cannot process this request with the available tools on {}.",
                    self.server_name
                );
            }

            // 2. Call MCP Tool
            match self.client.call_tool(&call.tool_name, call.arguments).await {
                Ok(output) => {
                    format!(
                        "✅ Tool '{}' on {} executed successfully:\n\n{}",
                        call.tool_name, self.server_name, output
                    )
                }
                Err(e) => format!("❌ MCP Tool execution failed: {}", e),
            }
        } else {
            "Error: Failed to decide on MCP tool execution.".to_string()
        }
    }

    fn category(&self) -> AgentCategory {
        // For now, we map all generic MCP agents to ACTION category,
        // or we could extend AgentCategory to include dynamic ones.
        AgentCategory::Action
    }
}
