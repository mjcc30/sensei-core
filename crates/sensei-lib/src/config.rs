use crate::errors::SenseiError;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

// --- Prompts Config (YAML) ---

#[derive(Debug, Deserialize)]
pub struct PromptsConfig {
    pub agents: HashMap<String, AgentConfig>,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub prompt: String,
}

pub fn load_prompts(path: &str) -> Result<PromptsConfig, SenseiError> {
    let mut file = File::open(path).map_err(|e| {
        SenseiError::Config(format!("Failed to open prompts file '{}': {}", path, e))
    })?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| SenseiError::Config(format!("Failed to read prompts file: {}", e)))?;

    let config: PromptsConfig = serde_yaml::from_str(&contents)
        .map_err(|e| SenseiError::Config(format!("Failed to parse YAML: {}", e)))?;
    Ok(config)
}

// --- MCP Settings (JSON) ---

#[derive(Debug, Deserialize)]
pub struct McpSettings {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

pub fn load_mcp_settings(path: &str) -> Result<McpSettings, SenseiError> {
    let mut file = File::open(path).map_err(|e| {
        SenseiError::Config(format!("Failed to open MCP settings '{}': {}", path, e))
    })?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| SenseiError::Config(format!("Failed to read MCP settings: {}", e)))?;

    let settings: McpSettings = serde_json::from_str(&contents)
        .map_err(|e| SenseiError::Config(format!("Failed to parse MCP JSON: {}", e)))?;
    Ok(settings)
}
