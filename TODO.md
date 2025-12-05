# 🦀 Sensei Core (v3) - Roadmap & TODO

> **Philosophy:** TDD First. Reliability. Performance.

## 🚨 Critical Missing Features (V2 Parity)
These features exist in Python v2 but are missing in Rust v3.

- [x] **Action Agent (Tools):**
    - [x] Create `ToolExecutorAgent` implementing `Agent` trait.
    - [x] Connect `NmapTool` and `SystemTool` to this agent.
    - [x] Update `Orchestrator` to dispatch `AgentCategory::Action` to this agent.
- [x] **RAG Ingestion:**
    - [x] Add CLI command `--add <file>` to ingest documents.
    - [x] Implement chunking and embedding logic in `sensei-server`.
    - [x] Store vectors in `sqlite-vec`.

## 🏗️ Phase 1: Foundation (Workspace & Architecture)
- [x] **Setup Workspace:** Convert project to Rust Workspace (`server`, `client`, `common`).
- [x] **Shared Types:** Define `Message`, `Query`, `Response` in `sensei-common`.
- [x] **Server Skeleton:** Implement a basic `Axum` HTTP server in `sensei-server`.
- [x] **Client Skeleton:** Implement a basic `Clap` CLI in `sensei-client` that queries the server.
- [x] **TDD:** Test Health Check endpoint.

## 🧠 Phase 2: The Brain (LLM & Memory)
- [x] **LLM Integration:** Move `genai` logic to `sensei-server` with **Smart Fallback** (auto-model selection).
- [x] **Memory Layer:** Implement SQLite connection using `sqlx` in `sensei-server`.
- [x] **TDD:** Test conversation/session persistence (CRUD).
- [x] **Vector Support:** Integrate `sqlite-vec` for RAG (Extension loaded).

## 🐝 Phase 3: The Swarm (Actors)
- [x] **Agent Trait:** Define the behavior of an Agent in Rust.
- [x] **Router:** Implement the intent classifier (`RouterAgent` with LLM) and **Query Optimizer** (iso-v2).
- [x] **Orchestrator:** Manage message passing between agents.
- [x] **Specialists:** Port Red/Blue/System agents (`SpecializedAgent`) using dynamic `prompts.yaml`.

## 🔌 Phase 4: Interfaces & Tools
- [ ] **MCP Server:** Expose tools via MCP protocol natively.
- [ ] **TUI (Optional):** Create a rich terminal UI with `ratatui` for the client.

## 🛡️ Quality Assurance
- [x] **CI/CD:** Rust Clippy, Rustfmt (prek) & Cargo Test workflows.
- [x] **Documentation:** READMEs and Doc-tests for `sensei-common`.
- [x] **Benchmarks:** V3 is **192x faster** than V2 (80ms vs 16s).
- [x] **Behavior Tests:** Router achieves **6/7** accuracy on v2 benchmark dataset.
