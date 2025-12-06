# 🧠 Sensei Server

The **Sensei Server** is the brain of the operation. It exposes a REST API to process user queries via LLMs and Tools.

## 🛠️ Tech Stack
*   **Framework:** `Axum`
*   **Runtime:** `Tokio`
*   **Database:** `SQLx` (SQLite)
*   **LLM:** `genai` (Gemini + Ollama)

## ⚙️ Configuration

The server relies on environment variables (or `.env` file):

| Variable | Description | Default |
| :--- | :--- | :--- |
| `GEMINI_API_KEY` | Google Gemini API Key | (Required) |
| `SENSEI_LISTEN_ADDR` | Address to bind to | `0.0.0.0:3000` |
| `SENSEI_PROMPTS_PATH` | Path to agent personas | `prompts.yaml` |
| `SENSEI_MCP_CONFIG` | Path to MCP tools config | `mcp_settings.json` |
| `OLLAMA_MODEL` | Local model for failover | (None) |

### Unix Domain Sockets (UDS)
To use a secure Unix socket instead of TCP:
```bash
export SENSEI_LISTEN_ADDR="unix:///tmp/sensei.sock"
```

## 🔄 Hot Reloading
The server automatically watches `mcp_settings.json`.
*   **Add a server:** Add an entry to the JSON. Sensei will spawn the new agent instantly.
*   **Remove a server:** Remove the entry. Sensei will unload the agent.

## 🔌 API Endpoints

*   `GET /health`: Health check.
*   `POST /v1/ask`: Main chat endpoint (supports `x-session-id`).
*   `POST /v1/feedback/correct`: RLHF endpoint to correct routing mistakes.
*   `POST /v1/knowledge/add`: Ingest documents for RAG.