use clap::Parser;
use reqwest::Client;
use sensei_common::{AskRequest, AskResponse};
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Question to ask Sensei
    #[arg(short, long)]
    ask: String,

    /// Server URL (e.g., http://localhost:3000)
    #[arg(short, long, default_value = "http://127.0.0.1:3000")]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = Client::new();
    let url = format!("{}/v1/ask", args.url.trim_end_matches('/'));

    // Prepare request using shared type
    let request = AskRequest {
        prompt: args.ask.clone(),
    };

    println!("Sending request to {}...", url);

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?;

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