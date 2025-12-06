# 🔌 Sensei MCP Server

This binary exposes the entire **Sensei Core** intelligence as a **Model Context Protocol (MCP)** server.

This allows you to use Sensei (and its tools like Nmap, RAG, System) directly inside:
*   **Claude Desktop**
*   **Cursor**
*   **Zed**

## 🚀 Installation with Claude Desktop

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sensei": {
      "command": "/absolute/path/to/target/release/sensei-mcp",
      "args": [],
      "env": {
        "DATABASE_URL": "sqlite:///absolute/path/to/sensei.db?mode=rwc",
        "GEMINI_API_KEY": "your_key"
      }
    }
  }
}
```

## 🛠️ Capabilities

*   **Tools:**
    *   `nmap`: Execute network scans.
    *   `system_diagnostic`: Check server health.
*   **Resources:**
    *   `sensei://knowledge/...`: Access documents stored in Sensei's RAG memory.
