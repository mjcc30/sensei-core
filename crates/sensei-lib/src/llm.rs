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

pub struct LlmClient {
    client: Client,
    model_config: String,
}

pub const MODEL_CHAT_FAST: &str = "gemini-2.5-flash";
pub const MODEL_CHAT_SMART: &str = "gemini-3-pro-preview";
pub const MODEL_CHAT_DEFAULT: &str = MODEL_CHAT_FAST;
pub const MODEL_EMBEDDING: &str = "gemini-embedding-001";

const MODELS_PREFERENCE: &[&str] = &[
    MODEL_CHAT_FAST,
    MODEL_CHAT_SMART,
    "gemini-2.5-pro",
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
            .map_err(|e| SenseiError::Config(serde_yaml::Error::custom(e.to_string())))?;
        // Need to wrap env error or change Config error type. Using generic Llm error for now.
        // Actually map_err(|_| SenseiError::Llm("GEMINI_API_KEY not set".into()))

        // Resolve model name if "auto"
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
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
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

        let models: Vec<&str> = if self.model_config == "auto" {
            MODELS_PREFERENCE.to_vec()
        } else {
            vec![self.model_config.as_str()]
        };

        let mut last_error = None;

        for model in models {
            use genai::chat::ChatOptions;
            let options = ChatOptions::default().with_temperature(0.7);

            match self
                .client
                .exec_chat(model, chat_req.clone(), Some(&options))
                .await
            {
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

        Err(SenseiError::Llm(format!(
            "All Gemini models failed. Last error: {:?}",
            last_error
        )))
    }
}

// Helper to fix serde_yaml import error in generate_raw
use serde::de::Error as SerdeError;
