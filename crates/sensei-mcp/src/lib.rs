use sensei_lib::memory::MemoryStore;
use sensei_lib::tools::Tool;
use sensei_lib::tools::nmap::NmapTool;
use sensei_lib::tools::system::SystemTool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

// --- JSON-RPC Types ---

#[derive(Deserialize, Debug)]
pub struct JsonRpcRequest {
    // Prefix with _ to indicate intentional unused field (parsed but ignored logic-wise)
    pub _jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

// --- MCP Types ---

#[derive(Serialize)]
struct ToolDescription {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Serialize)]
struct ResourceDescription {
    uri: String,
    name: String,
    mime_type: Option<String>,
}

// --- Server State ---

pub struct McpServer {
    memory: MemoryStore,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl McpServer {
    pub async fn new(db_url: &str) -> anyhow::Result<Self> {
        let memory = MemoryStore::new(db_url).await?;
        // Run migration to ensure DB schema is ready
        memory.migrate().await?;

        let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();

        // Register Tools
        let nmap = NmapTool;
        tools.insert(nmap.name().to_string(), Box::new(nmap));

        let system = SystemTool;
        tools.insert(system.name().to_string(), Box::new(system));

        Ok(Self { memory, tools })
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id.clone();
        let result = match req.method.as_str() {
            "initialize" => self.handle_initialize(req.params.clone()).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(req.params.clone()).await,
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resources_read(req.params.clone()).await,
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method '{}' not found", req.method),
            }),
        };

        match result {
            Ok(res) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(res),
                error: None,
            },
            Err(err) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(err),
            },
        }
    }

    async fn handle_initialize(&self, _params: Option<Value>) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "protocolVersion": "0.1.0",
            "server": {
                "name": "sensei-mcp",
                "version": "0.1.0"
            },
            "capabilities": {
                "tools": {},
                "resources": {}
            }
        }))
    }

    async fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let tools_list = vec![
            ToolDescription {
                name: "nmap".to_string(),
                description: "Run a network scan on a target.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "IP or Hostname" }
                    },
                    "required": ["target"]
                }),
            },
            ToolDescription {
                name: "system_diagnostic".to_string(),
                description: "Run system checks (uptime, df, free).".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "enum": ["uptime", "disk", "memory", "whoami", "date"],
                            "description": "Diagnostic command to run"
                        }
                    },
                    "required": ["command"]
                }),
            },
        ];

        Ok(json!({ "tools": tools_list }))
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or(JsonRpcError {
            code: -32602,
            message: "Missing params".into(),
        })?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or(JsonRpcError {
                code: -32602,
                message: "Missing tool name".into(),
            })?;

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        if let Some(tool) = self.tools.get(name) {
            let arg_str = if name == "nmap" {
                arguments
                    .get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            } else if name == "system_diagnostic" {
                arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                return Err(JsonRpcError {
                    code: -32602,
                    message: "Unknown tool signature".into(),
                });
            };

            match tool.execute(&arg_str).await {
                Ok(output) => Ok(json!({ "content": [{ "type": "text", "text": output }] })),
                Err(e) => Err(JsonRpcError {
                    code: -32000,
                    message: e.to_string(),
                }),
            }
        } else {
            Err(JsonRpcError {
                code: -32601,
                message: format!("Tool {} not found", name),
            })
        }
    }

    async fn handle_resources_list(&self) -> Result<Value, JsonRpcError> {
        let docs = self
            .memory
            .list_documents()
            .await
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: e.to_string(),
            })?;

        let resources: Vec<ResourceDescription> = docs
            .into_iter()
            .map(|(id, snippet)| ResourceDescription {
                uri: format!("sensei://knowledge/{}", id),
                name: format!("Document #{} - {}...", id, snippet.replace('\n', " ")),
                mime_type: Some("text/plain".to_string()),
            })
            .collect();

        Ok(json!({ "resources": resources }))
    }

    async fn handle_resources_read(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or(JsonRpcError {
            code: -32602,
            message: "Missing params".into(),
        })?;
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or(JsonRpcError {
                code: -32602,
                message: "Missing uri".into(),
            })?;

        // Collapsed if using let chains (stable since 1.88)
        if let Some(id_str) = uri.strip_prefix("sensei://knowledge/") 
           && let Ok(id) = id_str.parse::<i64>() 
        {
            let content = self
                .memory
                .get_document(id)
                .await
                .map_err(|e| JsonRpcError {
                    code: -32603,
                    message: format!("Failed to read doc: {}", e),
                })?;

            return Ok(json!(@{
                "contents": [{ 
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": content
                }]
            }));
        }

        Err(JsonRpcError {
            code: -32602,
            message: "Resource not found or invalid URI".into(),
        })
    }
}