use anyhow::Result;
use async_trait::async_trait;

pub mod nmap;
pub mod system;

#[async_trait]
/// Trait for defining executable tools.
///
/// # Example
///
/// ```
/// use async_trait::async_trait;
/// use sensei_server::tools::Tool;
/// use anyhow::Result;
///
/// struct EchoTool;
///
/// #[async_trait]
/// impl Tool for EchoTool {
///     fn name(&self) -> &str { "echo" }
///     async fn execute(&self, args: &str) -> Result<String> {
///         Ok(format!("Echo: {}", args))
///     }
/// }
/// ```
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, args: &str) -> Result<String>;
}
