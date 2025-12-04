use crate::agents::Agent;
use crate::llm::Llm;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use std::sync::Arc;

pub struct SpecializedAgent {
    llm: Arc<dyn Llm>,
    category: AgentCategory,
    system_prompt: String,
}

impl SpecializedAgent {
    pub fn new(llm: Arc<dyn Llm>, category: AgentCategory, system_prompt: &str) -> Self {
        Self {
            llm,
            category,
            system_prompt: system_prompt.to_string(),
        }
    }
}

#[async_trait]
impl Agent for SpecializedAgent {
    async fn process(&self, input: &str) -> String {
        // Simple prompt concatenation for now.
        // Ideally, we should update Llm trait to handle ChatMessage vectors for proper System/User separation.
        let full_prompt = format!(
            "{}

User Query: {}",
            self.system_prompt, input
        );

        match self.llm.generate(&full_prompt).await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Agent {:?} Error: {}", self.category, e);
                "I encountered an error processing your request.".to_string()
            },
        }
    }

    fn category(&self) -> AgentCategory {
        self.category
    }
}
