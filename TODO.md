# 🦈 Sensei Core Roadmap

## ✅ Phase 3: Core Swarm & RAG (Completed)
**Goal:** Rewrite Python engine in Rust, achieve feature parity, and enable Swarm Intelligence.
...

## ✅ Phase 4: Modern Interfaces & Protocols (Completed)
**Goal:** Enhance user experience and interoperability.
...
- [x] **Advanced A2A Protocol:** `[DELEGATE: EXTENSION]` recursive loop implemented.

## 🔌 Phase 5: Advanced Protocol Features (Next Focus)
**Goal:** Match Gemini-CLI capabilities.

- [x] **Hot Reloading:** Watch `mcp_settings.json` and reload agents without restart. (Implemented in Phase 3 battle plan, moved here).
- [ ] **Centralized Tool Registry:** Abstract tools from agents for better Function Calling.
- [ ] **SSE Transport:** Support HTTP/SSE for remote MCP servers.

## 🛡️ Phase 6: Resilience & Sovereignty (In Progress)
**Goal:** Production-grade reliability and unrestricted local execution.

- [x] **Local Intelligence (Ollama):** Implemented `TieredLlmClient` with automatic failover to Ollama models.
- [ ] **Native Inference Engine:** Replace external Ollama API with in-process `candle` or `llama-cpp-rs` integration for zero-latency local execution.
- [ ] **Byzantine Consensus:** Multi-agent voting system.
- [ ] **Security Model:** MAC/ABAC implementation.
    - [ ] **Data Classification:** Tag ingested documents with levels (Unclassified, Confidential, Secret, Top Secret).
    - [ ] **Agent Clearance:** Assign security clearance levels to each Agent.
    - [ ] **Enforcement:** Modify `MemoryStore::search` to enforce "No Read Up".
- [ ] **User Authentication:**
    - [ ] Implement `SO_PEERCRED` verification on UDS (Owner only).
    - [ ] Add API Key/Token authentication for remote (HTTP) clients.
- [ ] **Dynamic Swarm:**
    - [ ] Allow defining new agents in `prompts.yaml` without recompiling.

## 🖥️ Phase 10: Interface & UX (Low Priority)
**Goal:** Deliver a "Cyberpunk" terminal experience.

- [ ] **TUI (Terminal User Interface):** Advanced `ratatui` interface.

## 🔧 Tech Debt
- [ ] **Embedding Migration:** Migrate to `gemini-embedding-001`.