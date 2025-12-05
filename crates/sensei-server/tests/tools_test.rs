use sensei_server::tools::Tool;
use sensei_server::tools::nmap::NmapTool;
use sensei_server::tools::system::SystemTool;

#[tokio::test]
async fn nmap_sanitization_works() {
    let tool = NmapTool;
    // Valid inputs
    assert!(tool.execute("localhost").await.is_ok());

    // Invalid inputs (injection attempts)
    assert!(tool.execute("localhost; ls").await.is_err());
    assert!(tool.execute("localhost && echo hacked").await.is_err());
    assert!(tool.execute("$(whoami)").await.is_err());
    assert!(tool.execute("`whoami`").await.is_err());
}

#[tokio::test]
async fn system_allowlist_works() {
    let tool = SystemTool;

    // Allowed commands
    assert!(tool.execute("memory").await.is_ok());
    assert!(tool.execute("uptime").await.is_ok());
    assert!(tool.execute("whoami").await.is_ok());

    // Disallowed commands
    assert!(tool.execute("reboot").await.is_err());
    assert!(tool.execute("ls -la").await.is_err());
    assert!(tool.execute("cat /etc/passwd").await.is_err());
    assert!(tool.execute("unknown_cmd").await.is_err());
}
