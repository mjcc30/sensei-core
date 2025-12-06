use thiserror::Error;

#[derive(Error, Debug)]
pub enum SenseiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("LLM error: {0}")]
    Llm(String), 

    #[error("Configuration error: {0}")]
    Config(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tool execution failed: {0}")]
    Tool(String),

    #[error("Agent error: {0}")]
    Agent(String),
}