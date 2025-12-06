# 🦀 Sensei Core (v3)

**Sensei Core** is a high-performance, distributed AI Agent Swarm written in Rust. It replaces the legacy Python engine with a memory-safe, concurrent architecture capable of RAG, Tool Execution, and Multi-Model Reasoning.

## ✨ Features

*   **🐝 Agent Swarm:** Specialized agents (Red Team, Blue Team, Cloud, Crypto) orchestrated by a Router.
*   **🧠 Smart Routing:** Automatically routes queries to the best agent (and model tier) based on intent.
*   **⚡ Multi-Model:** Uses **Gemini 2.5 Flash** for fast tasks and **Gemini 3 Pro (Preview)** for deep reasoning.
*   **📚 RAG (Retrieval Augmented Generation):** Ingest documents (`sqlite-vec`) and automatically retrieve context during conversation.
*   **🛠️ Tool Execution:** Agents can run system commands (`nmap`, `uptime`, `df`) securely via an allowlist.
*   **💾 Persistence:** SQLite storage for chat sessions and vector embeddings (3072 dims).
*   **🔒 Secure Transport:** Uses **Unix Domain Sockets (UDS)** by default on Linux/macOS for secure local communication.
*   **🤖 MCP Integration:** Includes a **Model Context Protocol (MCP)** server to connect Sensei with AI IDEs like Cursor or Claude Desktop.

## 🚀 Getting Started

### Prerequisites
*   Rust 1.75+
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
The server runs as a background daemon.
*   **Linux/macOS:** Defaults to Unix Socket (`/tmp/sensei.sock`).
*   **Windows:** Defaults to TCP (`127.0.0.1:3000`).

```bash
./target/release/sensei-server
```

You can force a specific address:
```bash
SENSEI_LISTEN_ADDR=0.0.0.0:3000 ./target/release/sensei-server
```

### 2. Ask Questions (CLI)
Use the lightweight client. It automatically detects the best connection method.

```bash
# Ask a question
./target/release/sensei-client --ask "How to secure a Docker container?"

# Explicitly target a socket or URL
./target/release/sensei-client --url unix:///tmp/sensei.sock --ask "Hello"
./target/release/sensei-client --url http://127.0.0.1:3000 --ask "Hello"
```

### 3. MCP Server (Claude/Cursor Integration)
To use Sensei as a "Brain" for Claude Desktop or Cursor:

1.  Configure your MCP client (e.g., `claude_desktop_config.json`):
    ```json
    {
      "mcpServers": {
        "sensei": {
          "command": "/absolute/path/to/sensei-core/target/release/sensei-mcp",
          "args": [],
          "env": {
            "DATABASE_URL": "sqlite:///absolute/path/to/sensei.db"
          }
        }
      }
    }
    ```
2.  Restart Claude/Cursor. Sensei's tools and memory are now available!

## 🏗️ Architecture

The project is organized as a Cargo Workspace with 5 crates:

*   **`sensei-lib`**: The Core Logic (Agents, RAG, Tools, DB). Shared by all components.
*   **`sensei-server`**: The HTTP/UDS API Server (Axum).
*   **`sensei-client`**: The CLI Tool (Reqwest/Hyper).
*   **`sensei-mcp`**: The MCP Server (Stdio/JSON-RPC).
*   **`sensei-common`**: Shared types and errors.

## 🧪 Testing

```bash
# Run Unit Tests
cargo test --workspace

# Run Benchmark (TCP vs UDS)
python3 scripts/bench_uds_vs_tcp.py
```

## 📄 License
MIT
