# 🗣️ Sensei Client

The **Sensei Client** is a lightweight, fast CLI tool that acts as the user interface. It sends requests to the Sensei Server.

## 🛠️ Tech Stack
*   **CLI Parser:** `Clap` (Command Line Argument Parser)
*   **HTTP Client:** `Reqwest`
*   **Async:** `Tokio`

## 🚀 Usage

```bash
sensei-client [OPTIONS] --ask <QUESTION>
```

### Options

| Flag | Short | Description | Default |
| :--- | :--- | :--- | :--- |
| `--ask` | `-a` | The question or prompt to send | (Required) |
| `--url` | `-u` | The Server URL | `http://127.0.0.1:3000` |
| `--help` | `-h` | Print help | |
| `--version` | `-V` | Print version | |

### Examples

**Basic Question:**
```bash
cargo run -p sensei-client -- --ask "Explain Ownership"
```

**Targeting a Remote Server:**
```bash
cargo run -p sensei-client -- --url "http://192.168.1.50:3000" --ask "System status"
```
