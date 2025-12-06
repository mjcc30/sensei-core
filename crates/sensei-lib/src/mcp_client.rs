use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

// Reuse types from sensei-common if possible, or define local protocol types
// Since sensei-common types are specific to Sensei API, we'll define generic JSON-RPC here
// or move generic JSON-RPC types to sensei-common later. For now, local is fine.

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonRpcError {
    code: i32,
    message: String,
}

pub struct McpClient {
    _child: Child,
    reader: Mutex<BufReader<tokio::process::ChildStdout>>,
    writer: Mutex<tokio::process::ChildStdin>,
    next_id: Mutex<u64>,
}

impl McpClient {
    pub async fn new(command: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Let stderr go to console for debugging
            .spawn()
            .context(format!(
                "Failed to spawn MCP server: {} {:?}",
                command, args
            ))?;

        let stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;

        Ok(Self {
            _child: child,
            reader: Mutex::new(BufReader::new(stdout)),
            writer: Mutex::new(stdin),
            next_id: Mutex::new(1),
        })
    }

    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = {
            let mut lock = self.next_id.lock().await;
            let id = *lock;
            *lock += 1;
            id
        };

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let mut json = serde_json::to_string(&req)?;
        json.push('\n'); // JSON-RPC over Stdio usually implies line-delimited JSON

        // Write Request
        {
            let mut writer = self.writer.lock().await;
            writer.write_all(json.as_bytes()).await?;
            writer.flush().await?;
        }

        // Read Response
        // Note: This is a simplified synchronous request-response model (blocking the reader).
        // A full implementation would have a background reader loop dispatching responses by ID.
        // For simple usage (one request at a time), this works.
        {
            let mut reader = self.reader.lock().await;
            let mut line = String::new();

            // Loop to skip logs or non-JSON lines if any leak to stdout (though they shouldn't)
            // or to handle notifications.
            loop {
                line.clear();
                let bytes = reader.read_line(&mut line).await?;
                if bytes == 0 {
                    bail!("MCP Server closed connection unexpectedly");
                }

                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&line) {
                    if resp.id == Some(id) {
                        if let Some(err) = resp.error {
                            bail!("MCP Error {}: {}", err.code, err.message);
                        }
                        return Ok(resp.result.unwrap_or(Value::Null));
                    } else {
                        // Ignore mismatched IDs or notifications for now
                        continue;
                    }
                } else {
                    // Not valid JSON or not a response, maybe a log line?
                    // tracing::debug!("Ignored MCP output: {}", line);
                }
            }
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let _res = self.send_request("initialize", Some(json!({ "protocolVersion": "0.1.0", "client": { "name": "sensei-client", "version": "0.1.0" }, "capabilities": {} }))).await?;

        // Optionally check protocol version in response
        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<Value>> {
        let res = self.send_request("tools/list", Some(json!({}))).await?;

        if let Some(tools) = res.get("tools").and_then(|t| t.as_array()) {
            Ok(tools.clone())
        } else {
            Ok(vec![])
        }
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String> {
        let res = self
            .send_request(
                "tools/call",
                Some(json!({ "name": name, "arguments": arguments })),
            )
            .await?;

        // Extract content text
        // Response format: { "content": [{ "type": "text", "text": "..." }] }
        if let Some(content) = res.get("content").and_then(|c| c.as_array()) {
            let mut output = String::new();
            for part in content {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    output.push_str(text);
                }
            }
            Ok(output)
        } else {
            // Fallback: return raw JSON if structure doesn't match standard
            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}
