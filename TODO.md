# 🦈 Sensei Core Roadmap

## ✅ Phase 3: Core Swarm & RAG (Completed)
**Goal:** Rewrite Python engine in Rust, achieve feature parity, and enable Swarm Intelligence.

- [x] **Project Scaffolding:** Cargo workspace, Axum server, Tokio runtime.
- [x] **LLM Engine:** Multi-Model Support & God Mode logic.
- [x] **Agents & Routing:** Router, Specialists, A2A Loop.
- [x] **RAG (Memory):** SQLite Persistence, Vector Store (`sqlite-vec`).
- [x] **Integration:** GitHub CI/CD, Blackfin OS support.

## 🚀 Phase 4: Modern Interfaces & Protocols (In Progress)
**Goal:** Enhance user experience and interoperability.

- [x] **Architecture Refactor:** Modular `sensei-lib` + `server` + `client` + `mcp` crate.
- [x] **Security:** Unix Domain Sockets (UDS) support.
- [x] **Performance:** SQLite Hyper-Tuning (12M ops/sec).
- [x] **Learning:** RLHF Loop (Semantic Router Cache correction).
- [ ] **Protocol Unification (MCP Client):**
    - [x] `McpAgent` wrapper to treat MCP servers as native agents.
    - [ ] Dynamic Tool Discovery & Routing.
- [ ] **Advanced MCP Features:**
    - [ ] **SSE Transport:** Support HTTP/SSE for remote MCP servers.
    - [ ] **Centralized Tool Registry:** Abstract tools from agents for better Function Calling.
    - [ ] **Hot Reloading:** Watch `mcp_settings.json` and reload agents without restart.

## 🚧 Phase 4.5: TUI & UX (Current Focus)
**Goal:** Deliver a "Cyberpunk" terminal experience.

- [ ] **TUI (Terminal User Interface):**
    - [ ] Replace simple CLI print with full `ratatui` interface.
    - [ ] Features: Streaming output, Markdown rendering, Input history, Status panels.
    - [ ] "Cyberpunk/Hacker" aesthetic matching Blackfin theme.
    - [ ] Integrate UDS transport into TUI mode logic.

## 🛡️ Phase 5: Resilience & Sovereignty
**Goal:** Production-grade reliability and unrestricted local execution.

- [ ] **Local Intelligence (Ollama):** Fallback to local uncensored models (Llama 3, Mistral) for God Mode commands.
- [ ] **Byzantine Consensus:** Multi-agent voting system.
- [ ] **Security Model:** MAC/ABAC implementation.
    - [ ] Data Classification (Confidential/Secret tags).
    - [ ] Agent Clearance levels.
    - [ ] No Read Up enforcement in MemoryStore.
- [ ] **User Authentication:**
    - [ ] Implement `SO_PEERCRED` verification on UDS (Owner only).
    - [ ] Add API Key/Token authentication for remote (HTTP) clients.
- [ ] **Dynamic Swarm:**
    - [ ] Allow defining new agents in `prompts.yaml` without recompiling.

## 🔧 Tech Debt
- [ ] **Embedding Migration:** Migrate to `gemini-embedding-001`.