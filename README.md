# 🦀 Sensei Core (v3)

**Sensei Core** is a high-performance, distributed AI Agent Swarm written in Rust. It serves as a local "AI Brain" capable of RAG (Retrieval Augmented Generation), secure Tool Execution, and swarming intelligence.

Now with **MCP (Model Context Protocol)** support for integration with Claude Desktop and Cursor!

## ✨ Features

*   **🐝 Agent Swarm:** Specialized agents (Red Team, Blue Team, Router) orchestrated to solve complex tasks.
*   **🔌 MCP Server:** Native integration with Claude Desktop & Cursor via `sensei-mcp`.
*   **🔒 Secure by Design:** Supports **Unix Domain Sockets (UDS)** for locked-down local communication.
*   **🧠 RAG Memory:** SQLite + `sqlite-vec` for storing and retrieving knowledge embeddings.
*   **⚡ Multi-Model:** Intelligent routing between Fast (Flash) and Smart (Pro) Gemini models.
*   **🛠️ Tools:** Safe execution of `nmap` scans and system diagnostics.

## 📦 Architecture

The project is divided into modular crates:

| Crate | Binary | Role |
| :--- | :--- | :--- |
| `sensei-server` | `sensei-server` | **The Brain.** HTTP/UDS Server hosting the Swarm and Memory. |
| `sensei-client` | `sensei-client` | **The Voice.** CLI tool to interact with the server. |
| `sensei-mcp` | `sensei-mcp` | **The Connector.** MCP-compliant server for IDE integration (Stdio). |
| `sensei-lib` | - | **The Core.** Shared business logic (Agents, DB, LLM). |

## 🚀 Getting Started

### Prerequisites
*   Rust 1.91+ (Edition 2024)
*   `sqlite3` & `libsqlite3-dev`
*   `nmap` (optional, for Action Agent)
*   Google Gemini API Key

### Installation

1.  **Clone & Build:**
    ```bash
    git clone https://github.com/mjcc30/sensei-core.git
    cd sensei-core
    cargo build --release
    ```

2.  **Configuration:**
    Create a `.env` file:
    ```env
    GEMINI_API_KEY=your_api_key_here
    DATABASE_URL=sqlite://sensei.db?mode=rwc
    # Optional: Force a specific model
    # GEMINI_MODEL=gemini-2.0-flash
    ```

## 🎮 Usage Guide

### 1. Standard Mode (HTTP)
Run the server on a TCP port (default 3000) and use the CLI.

```bash
# Start Server
./target/release/sensei-server &

# Ask Question
./target/release/sensei-client --ask "How to secure Docker?"
```

### 2. Secure Mode (Unix Domain Sockets)
**Recommended for Linux/macOS.** Prevents network access to the API.

```bash
# Start Server on Socket
export SENSEI_LISTEN_ADDR=unix:///tmp/sensei.sock
./target/release/sensei-server &

# Connect Client
./target/release/sensei-client --url unix:///tmp/sensei.sock --ask "Hello Secure World"
```

### 3. MCP Mode (Claude Desktop / Cursor)
Integrate Sensei directly into your AI workflow.

**Claude Desktop Configuration:**
Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sensei": {
      "command": "/absolute/path/to/sensei-core/target/release/sensei-mcp",
      "args": [],
      "env": {
        "GEMINI_API_KEY": "your_key",
        "DATABASE_URL": "sqlite:///absolute/path/to/sensei.db?mode=rwc"
      }
    }
  }
}
```

Once connected, you can ask Claude:
> *"Use Sensei to scan my local network for open ports."*
> *"Read the latest documentation ingested in Sensei memory."*

## 🧪 Development & Testing

We enforce high code quality and coverage.

```bash
# Run all tests (Unit, Integration, Doc)
cargo test --workspace

# Run Linter
cargo clippy --workspace
```

## 🗺️ Roadmap

- [x] Phase 1-3: Core Swarm, RAG, Rust Rewrite
- [x] Phase 4a: Modular Architecture (Lib/Server/Client)
- [x] Phase 4b: MCP Implementation & UDS Security
- [ ] Phase 5: Advanced Security Model (MAC/ABAC)
- [ ] Phase 6: TUI Polish & Streaming

## 📄 License
MIT