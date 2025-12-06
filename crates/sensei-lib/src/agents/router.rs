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
    system_prompt: String,
}

impl RouterAgent {
    pub fn new(llm: Arc<dyn Llm>, system_prompt: &str) -> Self {
        Self {
            llm,
            system_prompt: system_prompt.to_string(),
        }
    }

    pub async fn classify(&self, input: &str) -> RoutingDecision {
        let prompt = format!("{}\n\nQuery: \"{}\"", self.system_prompt, input);

        match self.llm.generate(&prompt).await {
            Ok(json_str) => {
                println!("DEBUG LLM RAW: {}", json_str);
                // Robust JSON extraction: find first '{' and last '}'
                let start = json_str.find('{').unwrap_or(0);
                let end = json_str.rfind('}').map(|i| i + 1).unwrap_or(json_str.len());
                let candidate = &json_str[start..end];

                if let Ok(resp) = serde_json::from_str::<RouterResponse>(candidate) {
                    RoutingDecision {
                        category: resp.category,
                        query: resp.enhanced_query.unwrap_or(input.to_string()),
                    }
                } else {
                    eprintln!(
                        "Failed to parse router JSON. Raw: '{}', Candidate: '{}'",
                        json_str, candidate
                    );
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
