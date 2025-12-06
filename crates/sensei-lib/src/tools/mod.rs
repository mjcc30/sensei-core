use crate::errors::SenseiError;
use async_trait::async_trait;

pub mod nmap;
pub mod system;

#[async_trait]
/// Trait for defining executable tools.
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, args: &str) -> Result<String, SenseiError>;
}
