use crate::errors::SenseiError;
use crate::tools::Tool;
use async_trait::async_trait;
use std::env;
use std::process::Command;

pub struct NmapTool;

#[async_trait]
impl Tool for NmapTool {
    fn name(&self) -> &str {
        "nmap"
    }

    async fn execute(&self, target: &str) -> Result<String, SenseiError> {
        // Basic input sanitization to prevent command injection
        if target.contains(';')
            || target.contains('&')
            || target.contains('|')
            || target.contains('$')
            || target.contains('`')
            || target.contains('"')
            || target.contains('\'')
            || target.contains('(')
            || target.contains(')')
        {
            return Err(SenseiError::Tool(
                "Invalid characters in target name. Please provide a valid hostname or IP address.".into()
            ));
        }

        // Check if nmap is available in PATH or use provided path
        let nmap_path = env::var("SYSTEM_NMAPPATH").unwrap_or("nmap".to_string());

        let output = Command::new(nmap_path)
            .arg("-F") // Fast scan
            .arg(target)
            .output()
            .map_err(|e| SenseiError::Tool(format!("Failed to execute nmap command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SenseiError::Tool(format!("Nmap command failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}