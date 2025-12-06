use thiserror::Error;

#[derive(Error, Debug)]
pub enum SenseiError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Configuration IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration Parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Tool execution error: {0}")]
    Tool(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
