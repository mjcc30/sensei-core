use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct PromptsConfig {
    pub agents: HashMap<String, AgentConfig>,
}

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub prompt: String,
}

pub fn load_prompts(path: &str) -> Result<PromptsConfig> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: PromptsConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
