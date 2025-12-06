# 🗣️ Sensei Client

The **Sensei Client** is a lightweight, fast CLI tool that acts as the user interface. It sends requests to the Sensei Server.

## 🛠️ Tech Stack
*   **CLI Parser:** `Clap`
*   **HTTP Client:** `Reqwest` + `Hyper` (for UDS)
*   **Async:** `Tokio`

## 🚀 Usage

```bash
sensei-client [OPTIONS] --ask <QUESTION>
```

### Options

| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--ask` | `-a` | The question or prompt to send | (Optional) |
| `--url` | `-u` | Server URL (HTTP or UNIX) | `http://127.0.0.1:3000` |

### Examples

**Basic Question:**
```bash
sensei-client --ask "Explain Ownership"
```

**Secure Mode (Unix Socket):**
```bash
sensei-client --url "unix:///tmp/sensei.sock" --ask "System status"
```

**Ingest Document (RAG):**
```bash
sensei-client add secret_plans.txt
```