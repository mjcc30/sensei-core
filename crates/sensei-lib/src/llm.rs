use crate::errors::SenseiError;
use async_trait::async_trait;
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use genai::embed::EmbedRequest;
use serde_json::{Value, json};
use std::env;

#[async_trait]
pub trait Llm: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String, SenseiError>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>, SenseiError>;
    async fn generate_raw(&self, prompt: &str) -> Result<String, SenseiError> {
        self.generate(prompt).await
    }
}

pub const MODEL_CHAT_FAST: &str = "gemini-2.5-flash";
pub const MODEL_CHAT_SMART: &str = "gemini-3-pro-preview";
pub const MODEL_CHAT_DEFAULT: &str = MODEL_CHAT_FAST;
pub const MODEL_EMBEDDING: &str = "gemini-embedding-001";

// --- Gemini Implementation ---

pub struct GeminiClient {
    client: Client,
    model_config: String,
}

impl GeminiClient {
    pub fn new(model: &str) -> Self {
        let client = Client::default();
        Self {
            client,
            model_config: model.to_string(),
        }
    }
}

#[async_trait]
impl Llm for GeminiClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, SenseiError> {
        let req = EmbedRequest::new(text.to_string());
        let response = self
            .client
            .exec_embed(MODEL_EMBEDDING, req, None)
            .await
            .map_err(|e| SenseiError::Llm(e.to_string()))?;

        if let Some(embedding) = response.embeddings.first() {
            Ok(embedding.vector.clone())
        } else {
            Err(SenseiError::Llm("No embedding generated".to_string()))
        }
    }

    async fn generate_raw(&self, prompt: &str) -> Result<String, SenseiError> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| SenseiError::Config("GEMINI_API_KEY must be set".to_string()))?;

        let model_name = if self.model_config == "auto" {
            MODEL_CHAT_DEFAULT
        } else {
            &self.model_config
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_name, api_key
        );

        let body = json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "safetySettings": [
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE" }
            ]
        });

        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SenseiError::Llm(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(SenseiError::Llm(format!(
                "Gemini REST Error: {}",
                error_text
            )));
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| SenseiError::Llm(e.to_string()))?;
        if let Some(text) = json.pointer("/candidates/0/content/parts/0/text") {
            Ok(text.as_str().unwrap_or("").to_string())
        } else {
            Err(SenseiError::Llm(format!(
                "No content generated (Blocked?): {:?}",
                json
            )))
        }
    }

    async fn generate(&self, prompt: &str) -> Result<String, SenseiError> {
        let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);
        use genai::chat::ChatOptions;
        let options = ChatOptions::default().with_temperature(0.7);

        let model = if self.model_config == "auto" {
            MODEL_CHAT_DEFAULT
        } else {
            &self.model_config
        };

        match self.client.exec_chat(model, chat_req, Some(&options)).await {
            Ok(response) =>
            {
                #[allow(deprecated)]
                Ok(response
                    .content_text_as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Err(e) => Err(SenseiError::Llm(format!(
                "Gemini model '{}' failed: {}",
                model, e
            ))),
        }
    }
}

// --- Ollama Implementation ---

pub struct OllamaClient {
    client: Client,
    model: String,
}

impl OllamaClient {
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::default(),
            model: format!("ollama/{}", model), // genai expects "ollama/modelname"
        }
    }
}

#[async_trait]
impl Llm for OllamaClient {
    async fn generate(&self, prompt: &str) -> Result<String, SenseiError> {
        let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);
        // Ollama usually runs local, so no special auth needed by default in genai
        match self.client.exec_chat(&self.model, chat_req, None).await {
            Ok(response) =>
            {
                #[allow(deprecated)]
                Ok(response
                    .content_text_as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Err(e) => Err(SenseiError::Llm(format!(
                "Ollama model '{}' failed: {}",
                self.model, e
            ))),
        }
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>, SenseiError> {
        // TODO: Implement local embedding via Ollama if needed
        Err(SenseiError::Llm(
            "Ollama embedding not yet implemented".to_string(),
        ))
    }

    // Ollama is uncensored by design for many models, so raw = generate
    async fn generate_raw(&self, prompt: &str) -> Result<String, SenseiError> {
        self.generate(prompt).await
    }
}

// --- Tiered (Failover) Implementation ---

pub struct TieredLlmClient {
    primary: Box<dyn Llm>,
    secondary: Option<Box<dyn Llm>>,
}

impl TieredLlmClient {
    pub fn new(primary: Box<dyn Llm>, secondary: Option<Box<dyn Llm>>) -> Self {
        Self { primary, secondary }
    }
}

#[async_trait]
impl Llm for TieredLlmClient {
    async fn generate(&self, prompt: &str) -> Result<String, SenseiError> {
        match self.primary.generate(prompt).await {
            Ok(res) => Ok(res),
            Err(e) => {
                if let Some(ref sec) = self.secondary {
                    eprintln!(
                        "⚠️ Primary LLM failed ({}), failing over to Secondary...",
                        e
                    );
                    sec.generate(prompt).await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn generate_raw(&self, prompt: &str) -> Result<String, SenseiError> {
        match self.primary.generate_raw(prompt).await {
            Ok(res) => Ok(res),
            Err(e) => {
                if let Some(ref sec) = self.secondary {
                    eprintln!("⚠️ Primary LLM Raw gen failed ({}), failing over...", e);
                    sec.generate_raw(prompt).await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, SenseiError> {
        // Embeddings usually stick to primary for consistency
        self.primary.embed(text).await
    }
}

// Backward compatibility alias
pub type LlmClient = TieredLlmClient;
