# 🦈 Sensei Core Roadmap

## ✅ Phase 3: Core Swarm & RAG (Completed)
**Goal:** Rewrite Python engine in Rust, achieve feature parity, and enable Swarm Intelligence.

- [x] **Project Scaffolding:** Cargo workspace, Axum server, Tokio runtime.
- [x] **LLM Engine:**
    - [x] Client with `reqwest` / `genai`.
    - [x] Multi-Model Support (Flash for Router/Tools, Pro for Specialists).
    - [x] **God Mode:** `generate_raw` implementation to bypass safety filters (via `--raw`).
- [x] **Agents & Routing:**
    - [x] Router Agent (Intent Classification).
    - [x] Specialist Agents (Red, Blue, Cloud, etc.) with `prompts.yaml`.
    - [x] Action Agent (`ToolExecutor`) for Nmap/System commands.
    - [x] **A2A Loop:** ReAct pattern implemented in Orchestrator.
- [x] **RAG (Memory):**
    - [x] SQLite persistence (Sessions, Messages).
    - [x] Vector Store (`sqlite-vec` with 3072 dims).
    - [x] Ingestion CLI (`sensei add`).
- [x] **Release & Integration:**
    - [x] GitHub CI/CD (Release workflow, artifacts).
    - [x] **Blackfin OS Integration:** `recipe.yml`, Wrapper script, Systemd service.

---

## 🚧 Phase 4: Interfaces & Ecosystem (Current Focus)
**Goal:** Enhance user experience and interoperability.

- [ ] **TUI (Terminal User Interface):**
    - [ ] Replace simple CLI print with `ratatui` interface.
    - [ ] Features: Streaming output, Markdown rendering, Input history, Status panels.
    - [ ] "Cyberpunk/Hacker" aesthetic matching Blackfin theme.
- [ ] **MCP Server (Model Context Protocol):**
    - [ ] Create `crates/sensei-mcp`.
    - [ ] Expose Sensei's Memory (RAG) as MCP Resources.
    - [ ] Expose Sensei's Tools (Nmap, System) as MCP Tools.
    - [ ] Integration test with Claude Desktop or Cursor.

---

## 🔮 Future Vision (Phase 5: Security Model)
- [ ] **Access Control (MAC/ABAC):** Implement Bell-LaPadula model within the agent swarm.
    - [ ] **Data Classification:** Tag ingested documents with levels (Unclassified, Confidential, Secret, Top Secret).
    - [ ] **Agent Clearance:** Assign security clearance levels to each Agent.
    - [ ] **Enforcement:** Modify `MemoryStore::search` to enforce "No Read Up".
- [ ] **User Authentication:**
    - [ ] Switch transport to Unix Domain Sockets (UDS) with `chmod 700`.
    - [ ] Implement `SO_PEERCRED` verification (Owner only).
    - [ ] Add API Key/Token authentication for remote clients.
- [ ] **Dynamic Swarm:**
    - [ ] Allow defining new agents in `prompts.yaml` without recompiling (String-based categories).

---

## 🔧 Maintenance & Tech Debt
- [ ] **Embedding Migration:** Migrate from `text-embedding-004` to `gemini-embedding-001` before Jan 2026.
