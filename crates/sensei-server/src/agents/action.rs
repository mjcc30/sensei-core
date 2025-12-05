use crate::agents::Agent;
use crate::llm::Llm;
use crate::tools::Tool;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolExecutorAgent {
    llm: Arc<dyn Llm>,
    tools: HashMap<String, Box<dyn Tool>>,
    category: AgentCategory,
}

#[derive(Deserialize)]
struct ToolCall {
    tool_name: String,
    argument: String,
}

impl ToolExecutorAgent {
    pub fn new(llm: Arc<dyn Llm>, category: AgentCategory) -> Self {
        Self {
            llm,
            tools: HashMap::new(),
            category,
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    async fn decide_tool(&self, query: &str) -> Option<ToolCall> {
        let tools_list = self.tools.keys().cloned().collect::<Vec<_>>().join(", ");

        let prompt = format!(
            r#"            You are an autonomous Action Agent.
            Available Tools: [{}]
            Task: Analyze the user request and decide which tool to execute.
            User Request: "{}"
            Rules:
            - If the request matches a tool capability, output JSON: {{'tool_name': 'name', 'argument': 'value'}}
            - Tool "nmap": argument must be a target (IP/Host) e.g. "127.0.0.1".
            - Tool "system_diagnostic": argument must be one of: "uptime", "disk", "memory", "whoami", "date".
            - If NO tool matches or arguments are ambiguous, return JSON: {{'tool_name': 'none', 'argument': 'reason'}}

            Output strictly JSON.
            "#,
            tools_list, query
        );

        let response = self.llm.generate(&prompt).await.ok()?;

        // Robust JSON extraction
        let start = response.find('{').unwrap_or(0);
        let end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[start..end];

        serde_json::from_str::<ToolCall>(json_str).ok()
    }
}

#[async_trait]
impl Agent for ToolExecutorAgent {
    async fn process(&self, input: &str) -> String {
        // 1. Decide which tool to call
        if let Some(call) = self.decide_tool(input).await {
            if call.tool_name == "none" {
                return format!("I cannot perform this action: {}", call.argument);
            }

            // 2. Execute the tool
            if let Some(tool) = self.tools.get(&call.tool_name) {
                match tool.execute(&call.argument).await {
                    Ok(output) => {
                        format!(
                            "✅ Action executed successfully.\n\n**Tool Output:**\n```\n{}\n```",
                            output
                        )
                    }
                    Err(e) => format!("❌ Tool execution failed: {}", e),
                }
            } else {
                format!(
                    "Error: Tool '{}' selected by AI is not found in registry.",
                    call.tool_name
                )
            }
        } else {
            "Error: Failed to process action request (LLM Decision Failed).".to_string()
        }
    }

    fn category(&self) -> AgentCategory {
        self.category
    }
}
