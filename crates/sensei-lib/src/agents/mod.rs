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
use tokio::sync::RwLock;

#[async_trait]
pub trait Agent: Send + Sync {
    /// Process a user query and return a response.
    async fn process(&self, input: &str) -> String;

    /// Return the category/role of this agent.
    fn category(&self) -> AgentCategory;
}

pub struct Orchestrator {
    // RwLock allows concurrent reads and exclusive writes for Hot Reloading
    agents: RwLock<HashMap<AgentCategory, Arc<Box<dyn Agent>>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, agent: Box<dyn Agent>) {
        let cat = agent.category();
        println!("DEBUG: Registering agent for category {:?}", cat);
        self.agents.write().await.insert(cat, Arc::new(agent));
    }

    pub async fn unregister(&self, category: &AgentCategory) {
        println!("DEBUG: Unregistering agent for category {:?}", category);
        self.agents.write().await.remove(category);
    }

    pub async fn dispatch(&self, category: AgentCategory, input: &str) -> String {
        self.dispatch_loop(category, input, 3).await
    }

    #[async_recursion]
    async fn dispatch_loop(&self, category: AgentCategory, input: &str, depth: u8) -> String {
        // println!("DEBUG: Dispatching to {:?} (depth {})", category, depth);
        if depth == 0 {
            return "Error: Agent recursion limit reached (A2A loop detected).".to_string();
        }

        // Acquire read lock for agent lookup
        let agent = {
            let map = self.agents.read().await;
            if let Some(agent) = map.get(&category) {
                // println!("DEBUG: Found agent for {:?}", category);
                agent.clone()
            } else {
                // println!("DEBUG: Agent not found for {:?}. Fallback to Casual.", category);
                if let Some(casual) = map.get(&AgentCategory::Casual) {
                    casual.clone()
                } else {
                    return format!(
                        "No agent found for category {:?} and Casual fallback missing",
                        category
                    );
                }
            }
        }; // Lock released here

        let response = agent.process(input).await;

        // Optimized Protocol v2: [DELEGATE: CATEGORY] Payload
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"(?m)^\[DELEGATE:\s*(\w+)\]\s*(?s)(.*)$").unwrap());

        if let Some(caps) = re.captures(&response) {
            let target_cat_str = caps.get(1).map_or("", |m| m.as_str());
            let target_query = caps.get(2).map_or("", |m| m.as_str()).trim();

            let target_cat = match target_cat_str.to_uppercase().as_str() {
                "ACTION" => Some(AgentCategory::Action),
                "SYSTEM" => Some(AgentCategory::System),
                "RED" => Some(AgentCategory::Red),
                "BLUE" => Some(AgentCategory::Blue),
                "CLOUD" => Some(AgentCategory::Cloud),
                "CRYPTO" => Some(AgentCategory::Crypto),
                "OSINT" => Some(AgentCategory::Osint),
                "CASUAL" => Some(AgentCategory::Casual),
                name => Some(AgentCategory::Extension(name.to_lowercase())),
            };

            if let Some(cat) = target_cat {
                let observation = self.dispatch_loop(cat, target_query, depth - 1).await;

                let new_input = format!(
                    "{}\n\n[OBSERVATION from {}]\n{}",
                    input, target_cat_str, observation
                );
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
