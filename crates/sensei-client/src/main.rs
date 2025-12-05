use clap::{Parser, Subcommand};
use reqwest::Client;
use sensei_common::{AskRequest, AskResponse};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

mod tui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server URL (e.g., http://localhost:3000)
    #[arg(short, long, default_value = "http://127.0.0.1:3000")]
    url: String,

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
    let base_url = cli.url.trim_end_matches('/');
    let client = Client::new();

    // 1. Check --ask flag
    if let Some(prompt) = cli.ask {
        return print_ask(&client, base_url, &prompt).await;
    }

    // 2. Check Subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Ask { prompt } => return print_ask(&client, base_url, &prompt).await,
            Commands::Add { path } => return handle_add(&client, base_url, path).await,
        }
    }

    // 3. Check Direct Query
    if let Some(prompt) = cli.direct_query {
        return print_ask(&client, base_url, &prompt).await;
    }

    // 4. Default: TUI Mode
    // No arguments provided -> Interactive Mode
    tui::run_tui(client, base_url.to_string()).await?;

    Ok(())
}

async fn print_ask(client: &Client, base_url: &str, prompt: &str) -> Result<(), Box<dyn Error>> {
    println!("Sending request to {}/v1/ask...", base_url);
    match ask_api(client, base_url, prompt).await {
        Ok(content) => {
            println!("\n🥋 Sensei says:\n{}", content);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
    Ok(())
}

pub async fn ask_api(
    client: &Client,
    base_url: &str,
    prompt: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!("{}/v1/ask", base_url);
    let request = AskRequest {
        prompt: prompt.to_string(),
    };

    let response = client.post(&url).json(&request).send().await?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await?;
        return Err(format!("Server Error {}: {}", status, text).into());
    }

    let result: AskResponse = response.json().await?;
    Ok(result.content)
}

async fn handle_add(client: &Client, base_url: &str, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&path)?;
    let url = format!("{}/v1/knowledge/add", base_url);

    // JSON Payload matches AddDocumentRequest on server
    let body = json!({ "content": content });

    println!("📄 Ingesting document: {:?}", path);
    let response = client.post(&url).json(&body).send().await?;

    if response.status().is_success() {
        println!("✅ Document added to knowledge base.");
    } else {
        eprintln!("❌ Failed to add document: {}", response.status());
        let text = response.text().await?;
        eprintln!("Details: {}", text);
    }
    Ok(())
}
