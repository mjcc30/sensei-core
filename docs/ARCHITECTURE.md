# 🏛️ Sensei Core v3 Architecture

## Overview
Sensei v3 uses a **Modular Onion Architecture** to decouple the core logic from the various interfaces (CLI, HTTP, MCP).

## Components (Crates)

### 1. Sensei Lib (`crates/sensei-lib`) - *The Core*
*   **Role:** Contains all business logic. It is agnostic of the transport layer.
*   **Modules:**
    *   `agents`: Swarm orchestration, Router, Specialists.
    *   `memory`: SQLite persistence, Vector search (RAG).
    *   `llm`: GenAI integration (Gemini).
    *   `tools`: Tool implementations (Nmap, System).
    *   `errors`: Centralized error handling (`thiserror`).

### 2. Sensei Server (`crates/sensei-server`) - *The API*
*   **Role:** Exposes `sensei-lib` via a REST API.
*   **Tech:** `Axum`, `Tokio`.
*   **Transport:**
    *   **UDS (Unix Domain Sockets):** Default on Linux/macOS for local security (`/tmp/sensei.sock`).
    *   **TCP:** Default on Windows or via config (`0.0.0.0:3000`).

### 3. Sensei Client (`crates/sensei-client`) - *The Interface*
*   **Role:** Lightweight CLI for user interaction.
*   **Tech:** `Clap`, `Ratatui` (TUI), `Hyper`/`Reqwest`.
*   **Features:**
    *   Auto-detection of transport (UDS vs HTTP).
    *   Streaming responses.
    *   Interactive Terminal UI.

### 4. Sensei MCP (`crates/sensei-mcp`) - *The Agent Protocol*
*   **Role:** Implements the **Model Context Protocol (MCP)**.
*   **Transport:** Stdio (JSON-RPC 2.0).
*   **Purpose:** Allows external AI editors (Cursor, Claude) to use Sensei as a tool provider and knowledge base.

### 5. Sensei Common (`crates/sensei-common`) - *The Glue*
*   **Role:** Shared Data Types and Contracts.
*   **Content:** API Request/Response structs, Enums.

## Data Flow

### Scenario A: CLI Query (Local)
1.  User runs `sensei-client ask "Scan local network"`.
2.  **Client** detects Linux -> Connects to Unix Socket `/tmp/sensei.sock`.
3.  **Server** accepts connection (no TCP overhead).
4.  **Server** delegates to **Lib** (Router Agent).
5.  **Lib** decides "Action" -> Executes `NmapTool`.
6.  Response streams back to Client.

### Scenario B: Claude Desktop (MCP)
1.  Claude launches `sensei-mcp` via Stdio.
2.  Claude sends JSON-RPC `tools/call` ("nmap").
3.  **MCP Server** calls **Lib** (`NmapTool::execute`).
4.  Result returned as JSON-RPC response on stdout.

## Security Model
*   **Local Access:** Protected by file system permissions on the Unix Socket (`chmod 700`). Only the user who started the server can connect.
*   **Tool Sandbox:** Tools use a strict allowlist (no arbitrary shell execution).