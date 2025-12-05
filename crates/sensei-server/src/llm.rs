use anyhow::{Result, bail};
use async_trait::async_trait;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use std::env;

#[async_trait]
pub trait Llm: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
}

pub struct LlmClient {
    client: Client,
    model_config: String,
}

const MODELS_PREFERENCE: &[&str] = &[
    "gemini-2.5-flash",
    "gemini-2.0-flash",
    "gemini-1.5-flash",
    "gemini-1.5-flash-latest",
    "gemini-pro",
];

impl LlmClient {
    pub fn new(_api_key: String) -> Self {
        let client = Client::default();
        let model_config = env::var("GEMINI_MODEL").unwrap_or("auto".to_string());

        Self {
            client,
            model_config,
        }
    }
}

#[async_trait]
impl Llm for LlmClient {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);

        let models: Vec<&str> = if self.model_config == "auto" {
            MODELS_PREFERENCE.to_vec()
        } else {
            vec![self.model_config.as_str()]
        };

        let mut last_error = None;

        for model in models {
            // println!("Trying model: {}", model); // Debug log
            match self.client.exec_chat(model, chat_req.clone(), None).await {
                Ok(response) => {
                    #[allow(deprecated)]
                    return Ok(response
                        .content_text_as_str()
                        .unwrap_or_default()
                        .to_string());
                }
                Err(e) => {
                    eprintln!("⚠️ Model '{}' failed: {}. Trying next...", model, e);
                    last_error = Some(e);
                }
            }
        }

        bail!("All Gemini models failed. Last error: {:?}", last_error)
    }
}
