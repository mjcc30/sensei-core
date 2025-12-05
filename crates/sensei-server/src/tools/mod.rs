use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, args: &str) -> Result<String>;
}
