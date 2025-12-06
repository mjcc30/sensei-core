use anyhow::Result;
use async_trait::async_trait;
use sensei_common::AgentCategory;
use sensei_lib::agents::router::RouterAgent;
use sensei_lib::errors::SenseiError;
use sensei_lib::llm::Llm;
use sensei_lib::memory::MemoryStore;
use std::sync::Arc;

struct MockLlm;

#[async_trait]
impl Llm for MockLlm {
    async fn generate(&self, _prompt: &str) -> Result<String, SenseiError> {
        // Default wrong answer to prove cache correction works
        Ok(r#"{"category": "CASUAL", "enhanced_query": "Ignored"}"#.to_string())
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>, SenseiError> {
        // Mock constant embedding for test consistency
        Ok(vec![0.1; 3072])
    }
}

#[tokio::test]
async fn router_learns_from_correction() -> Result<()> {
    // 1. Setup
    let memory = MemoryStore::new("sqlite::memory:").await?;
    memory.migrate().await?;
    let llm = Arc::new(MockLlm);
    let router = RouterAgent::new(llm.clone(), Some(memory.clone()), "System Prompt");

    // 2. Initial Classification (Should match LLM default = CASUAL)
    let query = "Analyze SSH logs for brute force";
    let decision1 = router.classify(query).await;
    assert_eq!(decision1.category, AgentCategory::Casual);

    // 3. Apply Correction (Teach it that this is BLUE TEAM work)
    println!("Applying correction...");
    router.correct_decision(query, AgentCategory::Blue).await;

    // 4. Verify Cache Update (Should now return BLUE without asking LLM)
    // Note: Since MockLlm returns same embedding, search should hit.
    let decision2 = router.classify(query).await;
    
    assert_eq!(decision2.category, AgentCategory::Blue, "Router did not learn from correction!");
    
    // 5. Verify Update (Change mind to SYSTEM)
    println!("Applying update...");
    router.correct_decision(query, AgentCategory::System).await;
    
    let decision3 = router.classify(query).await;
    assert_eq!(decision3.category, AgentCategory::System, "Router did not update existing cache!");

    Ok(())
}
