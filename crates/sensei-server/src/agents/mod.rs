pub mod router;

use async_trait::async_trait;
use sensei_common::AgentCategory;
use std::collections::HashMap;
use std::sync::Arc;

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
        self.agents.insert(agent.category(), Arc::new(agent));
    }

    pub async fn dispatch(&self, category: AgentCategory, input: &str) -> String {
        if let Some(agent) = self.agents.get(&category) {
            agent.process(input).await
        } else {
            format!("No agent found for category {:?}", category)
        }
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}
