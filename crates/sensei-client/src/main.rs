use clap::{Parser, Subcommand};
use reqwest::Client;
use sensei_common::{AskRequest, AskResponse};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

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

    // Prioritize --ask flag for backward compatibility
    if let Some(prompt) = cli.ask {
        return handle_ask(&client, base_url, &prompt).await;
    }

    match cli.command {
        Some(Commands::Ask { prompt }) => handle_ask(&client, base_url, &prompt).await?,
        Some(Commands::Add { path }) => handle_add(&client, base_url, path).await?,
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
        }
    }

    Ok(())
}

async fn handle_ask(client: &Client, base_url: &str, prompt: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/v1/ask", base_url);
    let request = AskRequest {
        prompt: prompt.to_string(),
    };

    println!("Sending request to {}...", url);
    let response = client.post(&url).json(&request).send().await?;

    if !response.status().is_success() {
        eprintln!("❌ Server Error: {}", response.status());
        let text = response.text().await?;
        eprintln!("Details: {}", text);
        return Ok(());
    }

    let result: AskResponse = response.json().await?;
    println!("\n🥋 Sensei says:\n{}", result.content);
    Ok(())
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
