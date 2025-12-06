# 📦 Sensei Lib (Framework)

**Sensei Lib** is the engine behind Sensei Core. It allows developers to build custom AI Agent Systems in Rust.

## ✨ Capabilities

*   **Orchestrator:** Thread-safe, async agent dispatching with recursion support.
*   **MemoryStore:** High-performance SQLite wrapper with Vector Search (`sqlite-vec`) and Semantic Caching.
*   **LLM Client:** Tiered client supporting Google Gemini and Ollama with automatic failover.
*   **MCP Client:** Native support for connecting to Model Context Protocol servers.

## 🛠️ Usage Example

```rust
use sensei_lib::agents::{Orchestrator, Agent};
use sensei_lib::memory::MemoryStore;
use sensei_lib::llm::GeminiClient;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // 1. Setup Memory
    let memory = MemoryStore::new("sqlite::memory:").await.unwrap();
    
    // 2. Setup LLM
    let llm = Arc::new(GeminiClient::new("gemini-1.5-flash"));
    
    // 3. Setup Orchestrator
    let mut orchestrator = Orchestrator::new();
    
    // 4. Register Custom Agent
    // orchestrator.register(Box::new(MyAgent::new(...))).await;
    
    // 5. Dispatch
    let response = orchestrator.dispatch(AgentCategory::new("casual"), "Hello").await;
    println!("{}", response);
}
```
