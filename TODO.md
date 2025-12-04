# 🦀 Sensei Core (v3) - Roadmap & TODO

> **Philosophy:** TDD First. Reliability. Performance.

## 🏗️ Phase 1: Foundation (Workspace & Architecture)
- [x] **Setup Workspace:** Convert project to Rust Workspace (`server`, `client`, `common`).
- [x] **Shared Types:** Define `Message`, `Query`, `Response` in `sensei-common`.
- [x] **Server Skeleton:** Implement a basic `Axum` HTTP server in `sensei-server`.
- [x] **Client Skeleton:** Implement a basic `Clap` CLI in `sensei-client` that queries the server.
- [x] **TDD:** Test Health Check endpoint.

## 🧠 Phase 2: The Brain (LLM & Memory)
- [x] **LLM Integration:** Move `genai` logic to `sensei-server`.
- [x] **Memory Layer:** Implement SQLite connection using `sqlx` in `sensei-server`.
- [x] **TDD:** Test conversation/session persistence (CRUD).
- [x] **Vector Support:** Integrate `sqlite-vec` for RAG.

## 🐝 Phase 3: The Swarm (Actors)
- [x] **Agent Trait:** Define the behavior of an Agent in Rust.
- [x] **Router:** Implement the intent classifier (`RouterAgent` with LLM).
- [x] **Orchestrator:** Manage message passing between agents.
- [x] **Specialists:** Port Red/Blue/System agents (`SpecializedAgent`).

## 🔌 Phase 4: Interfaces & Tools
- [ ] **MCP Server:** Expose tools via MCP protocol natively.
- [ ] **TUI (Optional):** Create a rich terminal UI with `ratatui` for the client.

## 🛡️ Quality Assurance
- [x] **CI/CD:** Rust Clippy, Rustfmt (prek) & Cargo Test workflows.
- [x] **Documentation:** READMEs and Doc-tests for `sensei-common`.
- [ ] **Benchmarks:** Compare v2 vs v3 latency.
