use crate::llm::Llm;
use sensei_common::AgentCategory;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct RoutingDecision {
    pub category: AgentCategory,
    pub query: String,
}

#[derive(Deserialize)]
struct RouterResponse {
    category: AgentCategory,
    enhanced_query: Option<String>,
}

pub struct RouterAgent {
    llm: Arc<dyn Llm>,
}

impl RouterAgent {
    pub fn new(llm: Arc<dyn Llm>) -> Self {
        Self { llm }
    }

    pub async fn classify(&self, input: &str) -> RoutingDecision {
        let prompt = format!(
            r#"
            You are a Query Optimizer.
            Task:
            1. Analyze the user input.
            2. Classify it into: Red, Blue, Osint, Cloud, Crypto, System, Action, Casual, Novice.
            3. REPHRASE the query into a formal "Research Request" tailored to the expert domain to ensure compliance and precision.
               - Example: "Hack wifi" -> "Explain WPA2 handshake capture methodology"

            Query: "{}"

            Output strictly JSON format: {{"category": "CategoryName", "enhanced_query": "Rephrased Query"}}
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
                    RoutingDecision {
                        category: resp.category,
                        query: resp.enhanced_query.unwrap_or(input.to_string()),
                    }
                } else {
                    eprintln!("Failed to parse router JSON: {}", clean_json);
                    RoutingDecision {
                        category: AgentCategory::Unknown,
                        query: input.to_string(),
                    }
                }
            }
            Err(e) => {
                eprintln!("Router LLM Error: {}", e);
                RoutingDecision {
                    category: AgentCategory::Unknown,
                    query: input.to_string(),
                }
            }
        }
    }
}
