# 🦈 Sensei Core Roadmap

## ✅ Phase 3: Core Swarm & RAG (Completed)
**Goal:** Rewrite Python engine in Rust, achieve feature parity, and enable Swarm Intelligence.

- [x] **Project Scaffolding:** Cargo workspace, Axum server, Tokio runtime.
- [x] **LLM Engine:** Multi-Model Support (Flash/Pro) & God Mode (`--raw`).
- [x] **Agents & Routing:** Router, Specialists, Action/System Agents, ReAct Loop.
- [x] **RAG (Memory):** SQLite persistence & Vector Store (`sqlite-vec`).
- [x] **Release & Integration:** GitHub CI/CD & Blackfin OS Integration.

## ✅ Phase 3.5: Architecture & Security Hardening (Completed)
**Goal:** Modularize codebase and secure local transport.

- [x] **Modular Architecture:** Extracted core logic to `crates/sensei-lib` (Clean Architecture).
- [x] **Secure Transport (UDS):** Implemented Unix Domain Sockets (`unix:///tmp/sensei.sock`) for local client-server communication with `chmod 700`.
- [x] **Error Handling:** Migrated to `thiserror` for structured, robust error management in libraries.
- [x] **Testing:** Refactored unit and integration tests to match modular structure.

## ✅ Phase 4: MCP Integration (Completed)
**Goal:** Connect Sensei to the AI Ecosystem.

- [x] **MCP Server:** Created `crates/sensei-mcp` (Stdio transport).
- [x] **Tools Exposure:** Expose `nmap` and `system_diagnostic` as MCP Tools.
- [x] **Resources Exposure:** Expose RAG documents as MCP Resources (`sensei://knowledge/{id}`).

---

## 🚧 Phase 4.5: TUI & UX (Current Focus)
**Goal:** Deliver a "Cyberpunk" terminal experience.

- [ ] **TUI (Terminal User Interface):**
    - [ ] Replace simple CLI print with full `ratatui` interface.
    - [ ] Features: Streaming output, Markdown rendering, Input history, Status panels.
    - [ ] "Cyberpunk/Hacker" aesthetic matching Blackfin theme.
    - [ ] Integrate UDS transport into TUI mode logic.

---

## 🔮 Future Vision (Phase 5: Security Model)
- [ ] **Access Control (MAC/ABAC):** Implement Bell-LaPadula model within the agent swarm.
    - [ ] **Data Classification:** Tag ingested documents with levels (Unclassified, Confidential, Secret, Top Secret).
    - [ ] **Agent Clearance:** Assign security clearance levels to each Agent.
    - [ ] **Enforcement:** Modify `MemoryStore::search` to enforce "No Read Up".
- [ ] **User Authentication:**
    - [ ] Implement `SO_PEERCRED` verification on UDS (Owner only) for extra safety.
    - [ ] Add API Key/Token authentication for remote (HTTP) clients.
- [ ] **Dynamic Swarm:**
    - [ ] Allow defining new agents in `prompts.yaml` without recompiling.

---

## 🔧 Maintenance & Tech Debt
- [ ] **Embedding Migration:** Migrate from `text-embedding-004` to `gemini-embedding-001` before Jan 2026.