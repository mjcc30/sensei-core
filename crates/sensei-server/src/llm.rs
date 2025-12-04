use anyhow::{Context, Result};
use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};

pub struct LlmClient {
    client: Client,
    model: String,
}

impl LlmClient {
    pub fn new(_api_key: String) -> Self {
        // For this phase, we rely on the environment variable GEMINI_API_KEY being set.
        // 'genai' picks it up automatically.
        let client = Client::default();

        Self {
            client,
            model: "gemini-2.5-flash".to_string(),
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);

        let response = self
            .client
            .exec_chat(&self.model, chat_req, None)
            .await
            .context("Failed to execute chat via genai")?;

        // Using .content_text_as_str() despite deprecation warning for now (or update to .first_text())
        // To be clean:
        #[allow(deprecated)]
        Ok(response
            .content_text_as_str()
            .unwrap_or_default()
            .to_string())
    }
}
