use crate::tools::Tool;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::process::Command;

pub struct SystemTool;

#[async_trait]
impl Tool for SystemTool {
    fn name(&self) -> &str {
        "system_diagnostic"
    }

    async fn execute(&self, command_key: &str) -> Result<String> {
        // Strict allowlist of diagnostic commands
        let (cmd, args) = match command_key.trim() {
            "uptime" => ("uptime", vec![]),
            "disk" => ("df", vec!["-h"]),
            "memory" => ("free", vec!["-h"]),
            "whoami" => ("whoami", vec![]),
            "date" => ("date", vec![]),
            _ => bail!(
                "Unknown or disallowed diagnostic command: '{}'. Allowed: uptime, disk, memory, whoami, date",
                command_key
            ),
        };

        let output = Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run system command '{}': {}", cmd, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Command '{}' failed: {}", cmd, stderr);
        }

        // Truncate huge outputs to protect LLM context
        let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
        const MAX_LEN: usize = 4000;
        if stdout.len() > MAX_LEN {
            stdout.truncate(MAX_LEN);
            stdout.push_str("\n\n...[Output truncated by Sensei Safety Layer]...");
        }

        Ok(stdout)
    }
}
