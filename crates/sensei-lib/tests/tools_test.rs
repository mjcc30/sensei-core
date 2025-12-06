use sensei_lib::tools::Tool;
use sensei_lib::tools::nmap::NmapTool;
use sensei_lib::tools::system::SystemTool;

#[tokio::test]
async fn nmap_sanitization_works() {
    let tool = NmapTool;
    // Valid inputs
    // We expect these to fail execution because nmap is likely not installed in CI/Test env,
    // BUT we expect them to pass sanitization.
    // However, the tool returns Err if execution fails.
    // So we check the error message.

    let res = tool.execute("localhost").await;
    // It might succeed if nmap is there, or fail with "nmap not found".
    // But it should NOT fail with "Invalid characters".
    if let Err(e) = res {
        assert!(!e.to_string().contains("Invalid characters"));
    }

    // Invalid inputs (injection attempts)
    let res = tool.execute("localhost; ls").await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Invalid characters"));
}

#[tokio::test]
async fn system_allowlist_works() {
    let tool = SystemTool;

    // Allowed commands (might fail execution but pass allowlist)
    let res = tool.execute("uptime").await;
    if let Err(e) = res {
        assert!(!e.to_string().contains("Unknown or disallowed"));
    }

    // Disallowed commands
    let res = tool.execute("reboot").await;
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Unknown or disallowed")
    );
}
