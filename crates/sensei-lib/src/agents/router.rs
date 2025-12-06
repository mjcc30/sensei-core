use crate::llm::Llm;
use crate::memory::MemoryStore;
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
    memory: Option<MemoryStore>, // Option to allow testing/running without DB
    system_prompt: String,
}

impl RouterAgent {
    pub fn new(llm: Arc<dyn Llm>, memory: Option<MemoryStore>, system_prompt: &str) -> Self {
        Self {
            llm,
            memory,
            system_prompt: system_prompt.to_string(),
        }
    }

    fn check_fast_path(&self, input: &str) -> Option<RoutingDecision> {
        let input_lower = input.to_lowercase();

        // 1. Explicit Tool Call: "nmap ..." or "scan IP"
        // This is a "Core Heuristic" to ensure basic tools work flawlessly out of the box.
        // Future dynamic tools will rely on Semantic Cache (RLHF).
        if input_lower.contains("nmap")
            || (input_lower.starts_with("scan ") && input.chars().any(|c| c.is_numeric()))
        {
            return Some(RoutingDecision {
                category: AgentCategory::new("action"),
                query: input.to_string(),
            });
        }

        // 2. Explicit System Diagnostics
        if [
            "uptime",
            "whoami",
            "df -h",
            "free -h",
            "check disk",
            "check memory",
            "check ram",
        ]
        .iter()
        .any(|&cmd| input_lower.contains(cmd))
        {
            return Some(RoutingDecision {
                category: AgentCategory::new("system"),
                query: input.to_string(),
            });
        }

        None
    }

    pub async fn classify(&self, input: &str) -> RoutingDecision {
        // 0. Regex Fast Path (Zero Latency)
        if let Some(decision) = self.check_fast_path(input) {
            println!(
                "⚡ Regex Hit! Routing '{}' to {:?} (Saved ~1.5s)",
                input, decision.category
            );
            return decision;
        }

        // 1. Semantic Cache Lookup (Fast Path)
        if let Some(ref mem) = self.memory {
            // Generate embedding for query (Fast model embedding is cheap ~20ms)
            if let Ok(embedding) = self.llm.embed(input).await {
                // Threshold 0.1 means very close similarity
                let cache_hit = mem.search_router_cache(embedding.clone(), 0.1).await;

                // Try to resolve cache hit to a valid decision
                let cached_decision = if let Ok(Some((cat_str, enhanced))) = cache_hit {
                    serde_json::from_str::<AgentCategory>(&format!("\"{}\"", cat_str))
                        .ok()
                        .map(|category| (category, enhanced))
                } else {
                    None
                };

                if let Some((category, enhanced)) = cached_decision {
                    println!(
                        "⚡ Cache Hit! Routing '{}' to {:?} (Saved ~1s)",
                        input, category
                    );
                    return RoutingDecision {
                        category,
                        query: enhanced,
                    };
                }

                // If miss, proceed to LLM but keep embedding for caching later
                return self.classify_with_llm(input, Some(embedding)).await;
            }
        }

        // Fallback or No Cache
        self.classify_with_llm(input, None).await
    }

    async fn classify_with_llm(&self, input: &str, embedding: Option<Vec<f32>>) -> RoutingDecision {
        let prompt = format!("{}\n\nQuery: \"{}\"", self.system_prompt, input);

        match self.llm.generate(&prompt).await {
            Ok(json_str) => {
                println!("DEBUG LLM RAW: {}", json_str);
                let start = json_str.find('{').unwrap_or(0);
                let end = json_str.rfind('}').map(|i| i + 1).unwrap_or(json_str.len());
                let candidate = &json_str[start..end];

                if let Ok(resp) = serde_json::from_str::<RouterResponse>(candidate) {
                    let decision = RoutingDecision {
                        category: resp.category,
                        query: resp.enhanced_query.clone().unwrap_or(input.to_string()),
                    };

                    // Cache the result asynchronously if possible (but here we await for simplicity)
                    if let (Some(mem), Some(emb)) = (&self.memory, embedding) {
                        let cat_str = serde_json::to_string(&decision.category)
                            .unwrap()
                            .replace('"', "");
                        if let Err(e) = mem
                            .add_router_cache(input, &cat_str, &decision.query, emb)
                            .await
                        {
                            eprintln!("Failed to cache routing: {}", e);
                        }
                    }

                    decision
                } else {
                    eprintln!(
                        "Failed to parse router JSON. Raw: '{}', Candidate: '{}'",
                        json_str, candidate
                    );
                    RoutingDecision {
                        category: AgentCategory::new("unknown"),
                        query: input.to_string(),
                    }
                }
            }
            Err(e) => {
                eprintln!("Router LLM Error: {}", e);
                RoutingDecision {
                    category: AgentCategory::new("unknown"),
                    query: input.to_string(),
                }
            }
        }
    }

    /// Reinforcement Learning: Manually correct a routing decision.
    /// If a similar query exists in cache, it updates it. Otherwise, adds a new entry.
    pub async fn correct_decision(&self, input: &str, correct_category: AgentCategory) {
        let mem = match self.memory.as_ref() {
            Some(m) => m,
            None => return,
        };

        let embedding = match self.llm.embed(input).await {
            Ok(e) => e,
            Err(_) => return,
        };

        let cat_str = serde_json::to_string(&correct_category)
            .unwrap()
            .replace('"', "");

        // Try to update existing cache entry first
        match mem
            .update_router_cache_category(embedding.clone(), &cat_str)
            .await
        {
            Ok(true) => println!(
                "✅ Corrected router cache for '{}' -> {:?}",
                input, correct_category
            ),
            Ok(false) => {
                // If not found (or not similar enough), add as new knowledge
                println!(
                    "➕ Learned new routing for '{}' -> {:?}",
                    input, correct_category
                );
                // We use the raw input as the enhanced query for simplicity in correction
                let _ = mem
                    .add_router_cache(input, &cat_str, input, embedding)
                    .await;
            }
            Err(e) => eprintln!("Failed to correct cache: {}", e),
        }
    }
}
