use crate::agents::Agent;
use crate::llm::Llm;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use std::sync::Arc;

pub struct SpecializedAgent {
    llm: Arc<dyn Llm>,
    category: AgentCategory,
    system_prompt: String,
    master_prompt: Option<String>,
}

impl SpecializedAgent {
    pub fn new(
        llm: Arc<dyn Llm>,
        category: AgentCategory,
        system_prompt: &str,
        master_prompt: Option<String>,
    ) -> Self {
        Self {
            llm,
            category,
            system_prompt: system_prompt.to_string(),
            master_prompt,
        }
    }
}

#[async_trait]
impl Agent for SpecializedAgent {
    async fn process(&self, input: &str) -> String {
        let is_raw_mode = input.contains("--raw");

        // Select prompt based on mode
        let sys_prompt = if is_raw_mode && self.master_prompt.is_some() {
            self.master_prompt.as_ref().unwrap()
        } else {
            &self.system_prompt
        };

        let full_prompt = format!("{}\n\nUser Query: {}", sys_prompt, input);

        // Use raw generation (bypass filters) if --raw is requested AND allowed for this agent (Red)
        let result = if is_raw_mode && self.category == AgentCategory::Red {
            self.llm.generate_raw(&full_prompt).await
        } else {
            self.llm.generate(&full_prompt).await
        };

        match result {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Agent {:?} Error: {}", self.category, e);
                "I encountered an error processing your request.".to_string()
            }
        }
    }

    fn category(&self) -> AgentCategory {
        self.category
    }
}
