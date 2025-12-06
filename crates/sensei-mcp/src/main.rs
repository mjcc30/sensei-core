use anyhow::Result;
use dotenvy::dotenv;
use sensei_mcp::{JsonRpcRequest, McpServer};
use std::env;
use std::io::{self, BufRead};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let db_url = env::var("DATABASE_URL").unwrap_or("sqlite://sensei.db?mode=rwc".to_string());
    let server = Arc::new(McpServer::new(&db_url).await?);

    info!("🚀 Sensei MCP Server started. Listening on Stdio.");

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                continue;
            }
        };

        let server_clone = server.clone();
        tokio::spawn(async move {
            let res = server_clone.handle_request(req).await;
            let json = serde_json::to_string(&res).unwrap();
            println!("{}", json);
        });
    }

    Ok(())
}
