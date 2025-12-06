use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

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
    pub async fn new(
        command: &str,
        args: &[&str],
        env: Option<HashMap<String, String>>,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        if let Some(vars) = env {
            cmd.envs(vars);
        }

        let mut child = cmd.spawn().context(format!(
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
        json.push('\n');

        {
            let mut writer = self.writer.lock().await;
            writer.write_all(json.as_bytes()).await?;
            writer.flush().await?;
        }

        {
            let mut reader = self.reader.lock().await;
            let mut line = String::new();

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
                        continue;
                    }
                }
            }
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let _res = self.send_request("initialize", Some(json!({ "protocolVersion": "2024-11-05", "clientInfo": { "name": "sensei-client", "version": "0.1.0" }, "capabilities": {} }))).await?;
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

        if let Some(content) = res.get("content").and_then(|c| c.as_array()) {
            let mut output = String::new();
            for part in content {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    output.push_str(text);
                }
            }
            Ok(output)
        } else {
            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock MCP Server using Python (available in most envs)
    // Reads JSON-RPC line, replies with static response
    const MOCK_PY: &str = r#"
import sys, json

while True:
    line = sys.stdin.readline()
    if not line: break
    try:
        req = json.loads(line)
        method = req.get("method")
        msgid = req.get("id")
        
        if method == "initialize":
            res = {"jsonrpc": "2.0", "id": msgid, "result": {"protocolVersion": "0.1.0", "capabilities": {}}}
        elif method == "tools/list":
            res = {"jsonrpc": "2.0", "id": msgid, "result": {"tools": [{"name": "mock_tool"}]}}
        elif method == "tools/call":
            res = {"jsonrpc": "2.0", "id": msgid, "result": {"content": [{"type": "text", "text": "Mock Success"}]}}
        else:
            res = {}
            
        print(json.dumps(res), flush=True)
    except:
        pass
"#;

    #[tokio::test]
    async fn test_mcp_client_protocol() -> Result<()> {
        // Only run if python3 is available
        if std::process::Command::new("python3")
            .arg("--version")
            .output()
            .is_err()
        {
            println!("Skipping test: python3 not found");
            return Ok(());
        }

        let client = McpClient::new("python3", &["-c", MOCK_PY], None).await?;

        // 1. Initialize
        client.initialize().await?;

        // 2. List Tools
        let tools = client.list_tools().await?;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "mock_tool");

        // 3. Call Tool
        let output = client.call_tool("mock_tool", json!({})).await?;
        assert_eq!(output, "Mock Success");

        Ok(())
    }
}
