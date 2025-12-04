# 🦀 Sensei Core (v3) - Roadmap & TODO

> **Philosophy:** TDD First. Reliability. Performance.

## 🏗️ Phase 1: Foundation (Workspace & Architecture)
- [ ] **Setup Workspace:** Convert project to Rust Workspace (`server`, `client`, `common`).
- [ ] **Shared Types:** Define `Message`, `Query`, `Response` in `sensei-common`.
- [ ] **Server Skeleton:** Implement a basic `Axum` HTTP server in `sensei-server`.
- [ ] **Client Skeleton:** Implement a basic `Clap` CLI in `sensei-client` that queries the server.
- [ ] **TDD:** Test Health Check endpoint.

## 🧠 Phase 2: The Brain (LLM & Memory)
- [ ] **LLM Integration:** Move `genai` logic to `sensei-server`.
- [ ] **Memory Layer:** Implement SQLite connection using `sqlx` or `rusqlite` in `sensei-server`.
- [ ] **Vector Support:** Integrate `sqlite-vec` for RAG.
- [ ] **TDD:** Test conversation persistence.

## 🐝 Phase 3: The Swarm (Actors)
- [ ] **Agent Trait:** Define the behavior of an Agent in Rust.
- [ ] **Router:** Implement the intent classifier.
- [ ] **Orchestrator:** Manage message passing between agents (Tokio Channels).
- [ ] **Specialists:** Port Red/Blue/System agents.

## 🔌 Phase 4: Interfaces & Tools
- [ ] **MCP Server:** Expose tools via MCP protocol natively.
- [ ] **TUI (Optional):** Create a rich terminal UI with `ratatui` for the client.

## 🛡️ Quality Assurance
- [ ] **CI/CD:** Rust Clippy & Cargo Test workflows.
- [ ] **Benchmarks:** Compare v2 vs v3 latency.
