# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **RAG:** Placeholder for vector database integration (`sqlite-vec`).
- **Swarm:** Placeholder for Agent Actor Model implementation.

## [0.1.0] - 2025-01-01

### 🚀 Initial Release (v3 Skeleton)

This is the first pre-alpha release of the Rust rewrite of Sensei.

#### Features
- **Workspace Architecture:** Split into `sensei-server`, `sensei-client`, and `sensei-common`.
- **Server:**
    - `Axum` based REST API.
    - `POST /v1/ask` endpoint for LLM interaction.
    - `GET /health` endpoint.
    - **Persistence:** SQLite database with `sqlx` (Sessions & Messages tables).
    - **LLM:** Integrated `genai` crate supporting Gemini 2.5 Flash.
- **Client:**
    - `Clap` based CLI tool.
    - `--ask` argument to query the server.
- **Quality & Security:**
    - **CI/CD:** GitHub Actions for Audit, Clippy, and Tests.
    - **Pre-commit:** Integrated `prek` for local quality checks (fmt, clippy, test).
    - **Documentation:** Comprehensive READMEs and Doc-tests for shared types.

#### Tech Stack
- **Language:** Rust (2024 Edition).
- **Runtime:** Tokio.
- **Web:** Axum & Reqwest.
- **DB:** SQLx (SQLite).
- **AI:** genai.
