use clap::{Parser, Subcommand};
use sensei_common::{AskRequest, AskResponse};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

// HTTP / UDS handling
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;

mod tui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server URL (e.g., http://localhost:3000 or unix:///tmp/sensei.sock)
    #[arg(short, long)]
    url: Option<String>,

    /// Shortcut to ask a question
    #[arg(short, long)]
    ask: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Direct query (e.g. `sensei "hello"`)
    #[arg(index = 1)]
    direct_query: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ask a question to Sensei
    Ask { prompt: String },
    /// Add a document to Sensei's knowledge base (RAG)
    Add {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Determine default URL based on OS
    let default_url = if cfg!(unix) {
        "unix:///tmp/sensei.sock".to_string()
    } else {
        "http://127.0.0.1:3000".to_string()
    };

    let target_url = cli.url.unwrap_or(default_url);

    // 1. Check --ask flag
    if let Some(prompt) = cli.ask {
        return print_ask(&target_url, &prompt).await;
    }

    // 2. Check Subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Ask { prompt } => return print_ask(&target_url, &prompt).await,
            Commands::Add { path } => return handle_add(&target_url, path).await,
        }
    }

    // 3. Check Direct Query
    if let Some(prompt) = cli.direct_query {
        return print_ask(&target_url, &prompt).await;
    }

    // 4. Default: TUI Mode
    tui::run_tui(target_url).await?;

    Ok(())
}

async fn print_ask(url: &str, prompt: &str) -> Result<(), Box<dyn Error>> {
    println!("Sending request to {}/v1/ask...", url);
    match send_ask_request(url, prompt).await {
        Ok(content) => {
            println!("\n🥋 Sensei says:\n{}", content);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
    Ok(())
}

// Generic sender that switches between UDS (Hyper) and TCP (Reqwest)
pub async fn send_ask_request(
    base_url: &str,
    prompt: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let req_body = AskRequest {
        prompt: prompt.to_string(),
    };
    let json_body = serde_json::to_string(&req_body)?;

    if base_url.starts_with("unix://") {
        #[cfg(unix)]
        {
            let socket_path = base_url.trim_start_matches("unix://");
            let stream = UnixStream::connect(socket_path).await?;
            let io = TokioIo::new(stream);

            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    eprintln!("Connection failed: {:?}", err);
                }
            });

            let req = Request::builder()
                .method("POST")
                .uri("http://localhost/v1/ask") // UDS ignores host, path matters
                .header("Host", "localhost")
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(json_body)))?;

            let res = sender.send_request(req).await?;

            if !res.status().is_success() {
                return Err(format!("Server Error: {}", res.status()).into());
            }

            let body_bytes = res.collect().await?.to_bytes();
            let result: AskResponse = serde_json::from_slice(&body_bytes)?;
            Ok(result.content)
        }
        #[cfg(not(unix))]
        {
            Err("Unix sockets not supported".into())
        }
    } else {
        // Standard HTTP via Reqwest
        let client = reqwest::Client::new();
        let url = format!("{}/v1/ask", base_url.trim_end_matches('/'));
        let res = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(format!("Server Error: {}", res.status()).into());
        }
        let result: AskResponse = res.json().await?;
        Ok(result.content)
    }
}

async fn handle_add(base_url: &str, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&path)?;
    // Simplified JSON construction
    let json_body = serde_json::to_string(&json!({ "content": content }))?;

    if base_url.starts_with("unix://") {
        #[cfg(unix)]
        {
            let socket_path = base_url.trim_start_matches("unix://");
            let stream = UnixStream::connect(socket_path).await?;
            let io = TokioIo::new(stream);
            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
            tokio::task::spawn(async move { if let Err(_) = conn.await {} });

            let req = Request::builder()
                .method("POST")
                .uri("http://localhost/v1/knowledge/add")
                .header("Host", "localhost")
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(json_body)))?;

            let res = sender.send_request(req).await?;

            if res.status().is_success() {
                println!("✅ Document added.");
            } else {
                eprintln!("❌ Failed: {}", res.status());
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err("Unix sockets not supported".into())
        }
    } else {
        let client = reqwest::Client::new();
        let url = format!("{}/v1/knowledge/add", base_url.trim_end_matches('/'));
        let res = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;

        if res.status().is_success() {
            println!("✅ Document added.");
        } else {
            eprintln!("❌ Failed: {}", res.status());
        }
        Ok(())
    }
}
