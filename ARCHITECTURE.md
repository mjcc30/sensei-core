# 🏛️ Sensei Core v3 Architecture

## Overview
Sensei v3 uses a **Client-Server** architecture to decouple the user interface from the intelligence logic.

## Components

### 1. Sensei Server (`crates/sensei-server`)
*   **Role:** The "Brain". Runs as a background daemon or system service.
*   **Tech:** `Axum` (HTTP/REST), `Tokio` (Async Runtime), `SQLx` (Database).
*   **Responsibilities:**
    *   Holds the LLM Context (GenAI).
    *   Manages persistent memory (SQLite).
    *   Orchestrates Agents (Swarm).
    *   Exposes an API (e.g., `POST /ask`, `GET /history`).

### 2. Sensei Client (`crates/sensei-client`)
*   **Role:** The "Voice". Lightweight CLI.
*   **Tech:** `Clap` (Arguments), `Reqwest` (HTTP Client), `Ratatui` (UI - Future).
*   **Responsibilities:**
    *   Captures user input.
    *   Sends requests to the Server.
    *   Displays streamed responses.
    *   Fast startup (<50ms).

### 3. Common (`crates/sensei-common`)
*   **Role:** The "Language". Shared data structures.
*   **Responsibilities:**
    *   Defines request/response structs (`Query`, `AgentResponse`).
    *   Defines errors.

## Data Flow
1.  User types `sensei ask "Hello"` in CLI.
2.  **Client** sends POST `/v1/ask` to **Server** (localhost:3000).
3.  **Server** receives request, routes to **RouterAgent**.
4.  **RouterAgent** selects **Worker**.
5.  **Worker** generates response (via LLM or Tool).
6.  **Server** streams response back to **Client**.
7.  **Client** prints to stdout.
