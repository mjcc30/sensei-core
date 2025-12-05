use anyhow::{Result, bail};
use async_trait::async_trait;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use genai::embed::EmbedRequest;
use std::env;

#[async_trait]
pub trait Llm: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

pub struct LlmClient {
    client: Client,
    model_config: String,
}

pub const MODEL_CHAT_DEFAULT: &str = "gemini-2.5-flash";
pub const MODEL_EMBEDDING: &str = "gemini/gemini-embedding-001";

const MODELS_PREFERENCE: &[&str] = &[
    MODEL_CHAT_DEFAULT,
    "gemini-2.0-flash",
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

    pub fn new_with_model(_api_key: String, model: &str) -> Self {
        let client = Client::default();
        Self {
            client,
            model_config: model.to_string(),
        }
    }
}

#[async_trait]
impl Llm for LlmClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let req = EmbedRequest::new(text.to_string());
        let response = self.client.exec_embed(MODEL_EMBEDDING, req, None).await?;
        
        if let Some(embedding) = response.embeddings.first() {
            Ok(embedding.vector.clone())
        } else {
            bail!("No embedding generated")
        }
    }
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
