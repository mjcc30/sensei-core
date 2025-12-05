# 🦀 Sensei Core (v3)

**Sensei Core** is the next-generation, high-performance rewrite of the Sensei AI Mentor. Built with **Rust**, it aims to replace the v2 Python engine with a faster, safer, and more scalable distributed architecture.

## 🏛️ Architecture

Sensei v3 follows a **Client-Server** architecture (Workspace):

*   **🧠 Server (`crates/sensei-server`)**: A background daemon powered by `Axum` and `Tokio`. It manages the LLM connections (Gemini), persistence (SQLite), and Agent Swarm logic.
*   **🗣️ Client (`crates/sensei-client`)**: A lightweight CLI powered by `Clap`. It sends requests to the server and streams responses.
*   **📦 Common (`crates/sensei-common`)**: Shared types and logic protocol.

## 🚀 Quick Start

### Prerequisites
*   Rust 1.91.1 (Stable) or later & Cargo
*   Git
*   A Gemini API Key

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/mjcc30/sensei-core.git
    cd sensei-core
    ```

2.  **Configure Environment:**
    Create a `.env` file in the root or `crates/sensei-server/`:
    ```env
    GEMINI_API_KEY=your_api_key_here
    ```

3.  **Build:**
    ```bash
    cargo build --release
    ```

## 🛡️ Development & Security

We enforce strict quality standards using **prek** (a fast, Rust-native `pre-commit` alternative) and **GitHub Actions**.

### 1. Install Dev Tools
```bash
# Install prek (Better pre-commit in Rust)
cargo install prek

# Install Rust tools
cargo install cargo-audit
cargo install cargo-tarpaulin
```

### 2. Activate Hooks
To prevent committing bad code, install the git hooks:
```bash
prek install
```
Now, every `git commit` will automatically run:
*   `cargo fmt` (Formatting)
*   `cargo clippy` (Linting - Errors on warnings)
*   `cargo test` (Unit tests)

### 3. Run Checks Manually
```bash
prek run --all-files
```

## 🗺️ Roadmap

*   [x] **Phase 1:** Skeleton, Client-Server HTTP, GenAI Integration.
*   [ ] **Phase 2:** Persistence (SQLite), RAG (Vector DB).
*   [ ] **Phase 3:** Agent Swarm (Router, Red/Blue/System Agents).
*   [ ] **Phase 4:** MCP Protocol Implementation.

## 📄 License

MIT
