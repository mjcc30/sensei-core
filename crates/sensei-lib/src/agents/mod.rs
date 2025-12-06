pub mod action;
pub mod mcp_agent;
pub mod router;
pub mod specialists;

use async_recursion::async_recursion;
use async_trait::async_trait;
use regex::Regex;
use sensei_common::AgentCategory;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

#[async_trait]
pub trait Agent: Send + Sync {
    /// Process a user query and return a response.
    async fn process(&self, input: &str) -> String;

    /// Return the category/role of this agent.
    fn category(&self) -> AgentCategory;
}

pub struct Orchestrator {
    // Arc<Box<dyn Agent>> allows sharing agents safely
    agents: HashMap<AgentCategory, Arc<Box<dyn Agent>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent: Box<dyn Agent>) {
        let cat = agent.category();
        println!("DEBUG: Registering agent for category {:?}", cat);
        self.agents.insert(cat, Arc::new(agent));
    }

    pub async fn dispatch(&self, category: AgentCategory, input: &str) -> String {
        self.dispatch_loop(category, input, 3).await
    }

    #[async_recursion]
    async fn dispatch_loop(&self, category: AgentCategory, input: &str, depth: u8) -> String {
        // println!("DEBUG: Dispatching to {:?} (depth {})", category, depth);
        // println!("DEBUG: Available Agents: {:?}", self.agents.keys().collect::<Vec<_>>());
        if depth == 0 {
            return "Error: Agent recursion limit reached (A2A loop detected).".to_string();
        }

        let agent = if let Some(agent) = self.agents.get(&category) {
            println!("DEBUG: Found agent for {:?}", category);
            agent
        } else {
            println!(
                "DEBUG: Agent not found for {:?}. Fallback to Casual.",
                category
            );
            // Fallback to Casual if agent not found
            if let Some(casual) = self.agents.get(&AgentCategory::Casual) {
                casual
            } else {
                return format!(
                    "No agent found for category {:?} and Casual fallback missing",
                    category
                );
            }
        };

        let response = agent.process(input).await;
        // println!("DEBUG: Response from {:?}: '{}'", category, response);

        // Optimized Protocol v2: [DELEGATE: CATEGORY] Payload
        // (?s) allows dot to match newlines for the payload
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"(?m)^\[DELEGATE:\s*(\w+)\]\s*(?s)(.*)$").unwrap());

        if let Some(caps) = re.captures(&response) {
            let target_cat_str = caps.get(1).map_or("", |m| m.as_str());
            let target_query = caps.get(2).map_or("", |m| m.as_str()).trim();

            // Resolve category (including dynamic MCP extensions)
            // Note: This mapping is basic. Ideally we should have a dynamic registry lookup.
            let target_cat = match target_cat_str.to_uppercase().as_str() {
                "ACTION" => Some(AgentCategory::Action),
                "SYSTEM" => Some(AgentCategory::System),
                "RED" => Some(AgentCategory::Red),
                "BLUE" => Some(AgentCategory::Blue),
                "CLOUD" => Some(AgentCategory::Cloud),
                "CRYPTO" => Some(AgentCategory::Crypto),
                "OSINT" => Some(AgentCategory::Osint),
                "CASUAL" => Some(AgentCategory::Casual),
                // Support dynamic extensions by name (e.g. [DELEGATE: FILESYSTEM])
                name => Some(AgentCategory::Extension(name.to_lowercase())),
            };

            // println!("DEBUG: Resolved Target Cat: {:?}", target_cat);

            if let Some(cat) = target_cat {
                // 1. Execute Delegated Task
                let observation = self.dispatch_loop(cat, target_query, depth - 1).await;

                // 2. Feed Result back to Original Agent (ReAct Loop)
                // We use [OBSERVATION] block to match the new protocol style
                let new_input = format!(
                    "{}\n\n[OBSERVATION from {}]\n{}",
                    input, target_cat_str, observation
                );
                // Call the original agent again with the new context
                return self.dispatch_loop(category, &new_input, depth - 1).await;
            } else {
                return format!(
                    "Error: Agent attempted to delegate to unknown category '{}'",
                    target_cat_str
                );
            }
        }

        response
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}
