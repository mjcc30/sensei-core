use anyhow::Result;
use sensei_lib::mcp_client::McpClient;
use serde_json::json;
use std::path::PathBuf;

#[tokio::test]
async fn mcp_loopback_integration() -> Result<()> {
    // Locate the sensei-mcp binary
    // Assuming we are running from workspace root or crate root
    let bin_path = PathBuf::from("../../target/release/sensei-mcp");

    if !bin_path.exists() {
        println!(
            "⚠️ Skipping MCP test because binary not found at {:?}",
            bin_path
        );
        return Ok(());
    }

    // Connect to our own MCP Server
    let client = McpClient::new(bin_path.to_str().unwrap(), &[], None).await?;

    // 1. Initialize
    client.initialize().await?;

    // 2. List Tools
    let tools = client.list_tools().await?;
    println!("Tools found: {:?}", tools);

    assert!(tools.iter().any(|t| t["name"] == "nmap"));
    assert!(tools.iter().any(|t| t["name"] == "system_diagnostic"));

    // 3. Call Tool (System Diagnostic - safe)
    // We mock "uptime" call
    let response = client
        .call_tool("system_diagnostic", json!({ "command": "uptime" }))
        .await?;
    println!("Tool Output: {}", response);

    assert!(!response.is_empty());
    // Since uptime output varies, just checking non-empty is good enough for connection test.
    // The actual system tool execution is tested in unit tests.

    Ok(())
}
