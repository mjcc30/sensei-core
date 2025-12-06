# 🦈 Sensei Core Roadmap

## ✅ Phase 3: Core Swarm & RAG (Completed)
**Goal:** Rewrite Python engine in Rust, achieve feature parity, and enable Swarm Intelligence.

- [x] **Project Scaffolding:** Cargo workspace, Axum server, Tokio runtime.
- [x] **LLM Engine:** Multi-Model Support & God Mode logic.
- [x] **Agents & Routing:** Router, Specialists, A2A Loop.
- [x] **RAG (Memory):** SQLite Persistence, Vector Store (`sqlite-vec`).
- [x] **Integration:** GitHub CI/CD, Blackfin OS support.

## ✅ Phase 4: Modern Interfaces & Protocols (Completed)
**Goal:** Enhance user experience and interoperability.

- [x] **Architecture Refactor:** Modular `sensei-lib` + `server` + `client` + `mcp` crate.
- [x] **Security:** Unix Domain Sockets (UDS) support.
- [x] **Performance:** SQLite Hyper-Tuning (12M ops/sec).
- [x] **Learning:** RLHF Loop (Semantic Router Cache correction).
- [x] **Protocol Unification (MCP Client):** `McpAgent` wrapper and dynamic routing.
- [x] **Advanced A2A Protocol:** `[DELEGATE: EXTENSION]` recursive loop implemented.

## 🔌 Phase 5: Advanced Protocol Features (Next Focus)
**Goal:** Match Gemini-CLI capabilities.

- [ ] **Hot Reloading:** Watch `mcp_settings.json` and reload agents without restart.
- [ ] **Centralized Tool Registry:** Abstract tools from agents for better Function Calling.
- [ ] **SSE Transport:** Support HTTP/SSE for remote MCP servers.

## 🛡️ Phase 6: Resilience & Sovereignty
**Goal:** Production-grade reliability.

- [ ] **Local Intelligence (Ollama):** Fallback to local uncensored models (Llama 3, Mistral) for God Mode commands.
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
