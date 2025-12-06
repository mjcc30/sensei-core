# 🧠 Sensei Server

The **Sensei Server** is the brain of the operation. It exposes a REST API to process user queries via LLMs and Tools.

## 🛠️ Tech Stack
*   **Framework:** `Axum` (High performance, ergonomic, modular web framework)
*   **Runtime:** `Tokio` (Asynchronous runtime)
*   **LLM Engine:** `genai` (Universal LLM wrapper)
*   **Serialization:** `Serde`

## ⚙️ Configuration

The server relies on environment variables (or `.env` file):

| Variable | Description | Required |
| :--- | :--- | :--- |
| `GEMINI_API_KEY` | Google Gemini API Key | Yes |
| `PORT` | Listening port (Default: 3000) | No (Hardcoded for now) |

## 🔌 API Endpoints

### `GET /health`
Returns the status of the server.
*   **Response:** `200 OK`
    ```json
    { "status": "ok" }
    ```

### `POST /v1/ask`
Sends a prompt to the AI.
*   **Body:**
    ```json
    { "prompt": "Why is Rust memory safe?" }
    ```
*   **Response:**
    ```json
    { "content": "Rust guarantees memory safety through its ownership system..." }
    ```

## 📦 Usage

```bash
# Debug mode
cargo run -p sensei-server

# Release mode
cargo run -p sensei-server --release
```
