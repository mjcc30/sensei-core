use sensei_mcp::{JsonRpcRequest, McpServer};
use serde_json::json;

#[tokio::test]
async fn mcp_server_initializes_and_lists_tools() {
    // 1. Setup Server with In-Memory DB
    let server = McpServer::new("sqlite::memory:").await.unwrap();

    // 2. Test Initialize
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: Some(json!({})),
    };

    let init_res = server.handle_request(init_req).await;
    assert!(init_res.error.is_none());
    assert!(init_res.result.is_some());

    let result = init_res.result.unwrap();
    assert_eq!(result["server"]["name"], "sensei-mcp");

    // 3. Test Tools List
    let tools_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: Some(json!({})),
    };

    let tools_res = server.handle_request(tools_req).await;
    assert!(tools_res.error.is_none());

    let tools = tools_res.result.unwrap();
    let tools_array = tools["tools"].as_array().unwrap();
    assert!(tools_array.len() >= 2); // Nmap & System

    // Check for "nmap" tool
    let has_nmap = tools_array.iter().any(|t| t["name"] == "nmap");
    assert!(has_nmap);
}

#[tokio::test]
async fn mcp_server_handles_unknown_method() {
    let server = McpServer::new("sqlite::memory:").await.unwrap();

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(99)),
        method: "unknown/method".to_string(),
        params: None,
    };

    let res = server.handle_request(req).await;
    assert!(res.error.is_some());
    assert_eq!(res.error.unwrap().code, -32601); // Method not found
}
