# 🦀 Sensei Core (v3)

**Sensei Core** is a high-performance, distributed AI Agent Swarm written in Rust. It replaces the legacy Python engine with a memory-safe, concurrent architecture capable of RAG, Tool Execution, and Multi-Model Reasoning.

## ✨ Features

*   **🐝 Agent Swarm:** Specialized agents (Red Team, Blue Team, Cloud, Crypto) orchestrated by a Router.
*   **🧠 Smart Routing:** Automatically routes queries to the best agent (and model tier) based on intent.
*   **⚡ Multi-Model:** Uses **Gemini 2.5 Flash** for fast tasks and **Gemini 3 Pro (Preview)** for deep reasoning.
*   **📚 RAG (Retrieval Augmented Generation):** Ingest documents (`sqlite-vec`) and automatically retrieve context during conversation.
*   **🛠️ Tool Execution:** Agents can run system commands (`nmap`, `uptime`, `df`) securely via an allowlist.
*   **💾 Persistence:** SQLite storage for chat sessions and vector embeddings (3072 dims).

## 🚀 Getting Started

### Prerequisites
*   Rust 1.91.1+
*   `sqlite3` & `libsqlite3-dev`
*   `nmap` (optional, for Action Agent)
*   A Google Gemini API Key

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/mjcc30/sensei-core.git
    cd sensei-core
    ```

2.  **Configure Environment:**
    Create a `.env` file:
    ```env
    GEMINI_API_KEY=your_api_key_here
    DATABASE_URL=sqlite://sensei.db?mode=rwc
    ```

3.  **Build:**
    ```bash
    cargo build --release
    ```

## 🎮 Usage

### 1. Start the Server
The server acts as the central brain (Daemon).
```bash
./target/release/sensei-server
```

### 2. Ask Questions (CLI)
Use the lightweight client to interact.
```bash
# Standard query
./target/release/sensei-client ask "How to secure a Docker container?"

# Direct mode (shortcut)
./target/release/sensei-client "Explain buffer overflow"

# "God Mode" (Red Team Raw)
# Requires 'red_team' classification and '--raw' flag
./target/release/sensei-client ask "Write a C exploit for CVE-2024-XXXX --raw"
```

### 3. Ingest Knowledge (RAG)
Feed the brain with text files.
```bash
echo "The production database password is 'hunter2'" > secrets.txt
./target/release/sensei-client add secrets.txt

# Verify retrieval
./target/release/sensei-client ask "What is the db password?"
```

## ⚙️ Advanced Configuration

You can override system defaults via environment variables:

*   `GEMINI_MODEL`: Force a specific model (e.g., `gemini-2.0-flash-001`) globally, bypassing the tiered routing (Fast/Smart).
*   `SENSEI_PROMPTS_PATH`: Absolute path to a custom `prompts.yaml` (default: `./prompts.yaml`). Essential for production deployments (e.g., `/etc/sensei/prompts.yaml`).
*   `RUST_LOG`: Control logging verbosity (e.g., `debug`, `info`, `warn`).

### Configuration Files
*   [prompts.yaml](./prompts.yaml): Defines agent personas (system prompts) and router classification logic.
*   [.env.example](./.env.example): Template for environment variables.

## 🏗️ Architecture

*   **Server (`crates/sensei-server`):** Axum-based REST API. Manages lifecycle, DB connection pool, and LLM clients.
*   **Agents:**
    *   `RouterAgent`: Classifies intent (Red, Blue, System...) and optimizes queries.
    *   `SpecializedAgent`: Expert persona (prompt-engineered) utilizing Smart LLM.
    *   `ToolExecutorAgent`: Agent capable of calling `Tool` traits (Nmap, System).
*   **Memory:** SQLite with `sqlite-vec` extension for vector similarity search.

## 🧪 Testing & Quality

We enforce strict quality standards.

```bash
# Run Unit & Integration Tests
cargo test --workspace

# Run E2E Benchmarks (requires python3)
python3 scripts/benchmark.py      # Performance (v2 vs v3)
python3 scripts/test_rag.py       # RAG Capability
python3 scripts/validate_quality.py # Response Quality
```

## 🗺️ Roadmap

See [TODO.md](./TODO.md) for detailed progress.
- [x] Core V3 (Swarm, RAG, Tools)
- [ ] Phase 4: MCP Protocol Implementation
- [ ] Phase 4: TUI (Terminal UI)
- [ ] Phase 5: Security Model (MAC/ABAC)

## 📄 License
MIT
