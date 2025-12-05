use crate::tools::Tool;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::env;
use std::process::Command;

pub struct NmapTool;

#[async_trait]
impl Tool for NmapTool {
    fn name(&self) -> &str {
        "nmap"
    }

    async fn execute(&self, target: &str) -> Result<String> {
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
            bail!(
                "Invalid characters in target name. Please provide a valid hostname or IP address."
            );
        }

        // Check if nmap is available in PATH or use provided path
        let nmap_path = match env::var("SYSTEM_NMAPPATH") {
            Ok(path) => path,
            Err(_) => "nmap".to_string(), // Default to looking in PATH
        };

        // Execute nmap command
        // Note: For CI/safety, this might need to be simulated or run in a container.
        // For now, we assume nmap is available.
        let output = Command::new(nmap_path)
            .arg("-F") // Fast scan
            .arg(target)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute nmap command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Nmap command failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
