# 🦈 Sensei Core (Agent OS)

**Sensei Core** is a high-performance, modular AI Agent Orchestration framework written in Rust. It is designed to build **Agent Operating Systems** capable of running specialized swarms locally, securely, and at enterprise scale.

Initially built for Cybersecurity (Blackfin OS), it is now domain-agnostic.

## 🌟 Key Features

*   **🧠 Agent Swarm Architecture:** Orchestrate specialized agents (Rust-native or MCP-based) via a central brain.
*   **🔌 Protocol Unification:** Treats local Rust agents and external [MCP (Model Context Protocol)](https://modelcontextprotocol.io) servers identically.
*   **⚡ Enterprise Performance:**
    *   **12,000,000+ ops/sec** SQLite throughput (tuned).
    *   **< 5ms Routing Latency** via Semantic Caching (RLHF).
*   **🛡️ Sovereignty & Security:**
    *   **Local First:** Runs 100% offline with Ollama/Llama 3.
    *   **Secure Transport:** Uses Unix Domain Sockets (UDS) by default (`unix:///tmp/sensei.sock`).
    *   **Sandboxed Execution:** Tool execution is strictly controlled.
*   **🔄 Dynamic & Self-Healing:**
    *   **Hot Reloading:** Add/Remove MCP agents without restarting the server.
    *   **A2A Protocol:** Recursive Agent-to-Agent delegation (`[DELEGATE: AGENT]`).
    *   **Learning Loop:** Correct routing errors via API to teach the system.

## 🚀 Getting Started

### Prerequisites
*   Rust 1.80+
*   `sqlite3` & `libsqlite3-dev`
*   A Google Gemini API Key (or Ollama for local mode)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/mjcc30/sensei-core.git
    cd sensei-core
    ```

2.  **Configure:**
    Create a `.env` file:
    ```env
    GEMINI_API_KEY=your_api_key_here
    DATABASE_URL=sqlite://sensei.db?mode=rwc
    # Optional: Local Inference
    # OLLAMA_MODEL=llama3
    ```

3.  **Build:**
    ```bash
    cargo build --release
    ```

## 🎮 Usage

### 1. Start the Server (The Brain)
```bash
# Default (TCP 3000)
./target/release/sensei-server

# Secure Mode (Unix Socket)
SENSEI_LISTEN_ADDR=unix:///tmp/sensei.sock ./target/release/sensei-server
```

### 2. Use the Client (The Voice)
```bash
# Ask a question
./target/release/sensei-client --ask "Scan 192.168.1.1"

# Add knowledge (RAG)
./target/release/sensei-client add secrets.txt
```

### 3. Connect External Tools (MCP)
Create a `mcp_settings.json` file:
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/user"]
    }
  }
}
```
Sensei will automatically load this agent. You can then ask: *"List files in my home directory"*.

## 🏗️ Architecture

The project is organized as a Cargo Workspace:

*   **`crates/sensei-lib`**: The core logic (Orchestrator, Memory, LLM, Agents). Import this to build your own bot.
*   **`crates/sensei-server`**: The HTTP/UDS API server.
*   **`crates/sensei-client`**: The CLI tool.
*   **`crates/sensei-mcp`**: A standalone MCP Server implementation allowing Sensei to be used *by* Claude Desktop.
*   **`crates/sensei-common`**: Shared types.

## ⚙️ Configuration

*   **`prompts.yaml`**: Defines the persona of internal agents.
*   **`mcp_settings.json`**: Defines external MCP tools.

## 🧪 Performance & Quality

We enforce strict quality standards:
*   **Benchmarks:** `cargo run -p sensei-server --example bench_sqlite --release`
*   **QA:** `python3 scripts/validate_quality.py`
*   **Coverage:** Unit tests, Integration tests, and E2E flows.

## 📄 License
GNU GPLv3
