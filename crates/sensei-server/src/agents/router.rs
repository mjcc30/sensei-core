use crate::llm::Llm;
use sensei_common::AgentCategory;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct RouterResponse {
    category: AgentCategory,
}

pub struct RouterAgent {
    llm: Arc<dyn Llm>,
}

impl RouterAgent {
    pub fn new(llm: Arc<dyn Llm>) -> Self {
        Self { llm }
    }

    pub async fn classify(&self, input: &str) -> AgentCategory {
        let prompt = format!(
            r#"
            Classify the following query into one of these categories:
            Red, Blue, Osint, Cloud, Crypto, System, Action, Casual, Novice.

            Query: "{}"

            Output strictly JSON format: {{"category": "CategoryName"}}
            Example: {{"category": "Red"}}
            "#,
            input
        );

        match self.llm.generate(&prompt).await {
            Ok(json_str) => {
                let clean_json = json_str
                    .replace("```json", "")
                    .replace("```", "")
                    .trim()
                    .to_string();

                if let Ok(resp) = serde_json::from_str::<RouterResponse>(&clean_json) {
                    resp.category
                } else {
                    eprintln!("Failed to parse router JSON: {}", clean_json);
                    AgentCategory::Unknown
                }
            }
            Err(e) => {
                eprintln!("Router LLM Error: {}", e);
                AgentCategory::Unknown
            }
        }
    }
}
