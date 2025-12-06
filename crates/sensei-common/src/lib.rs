use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a health check response.
#[derive(Serialize, Deserialize, Debug)]
pub struct Health {
    pub status: String,
}

/// Represents a user request to the AI.
#[derive(Serialize, Deserialize, Debug)]
pub struct AskRequest {
    pub prompt: String,
}

/// Represents the AI's response.
#[derive(Serialize, Deserialize, Debug)]
pub struct AskResponse {
    pub content: String,
}

/// Generic Agent Category (Wrapper around String).
/// Allows dynamic categories like "RED", "STEAM", "KERNEL" without recompilation.
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(transparent)]
pub struct AgentCategory(pub String);

impl AgentCategory {
    pub fn new(name: &str) -> Self {
        // Normalize to lowercase for consistent routing keys
        Self(name.trim().to_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_uppercase())
    }
}

// Helper for common conversions
impl From<&str> for AgentCategory {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

// Manual Deserialize to enforce normalization
impl<'de> Deserialize<'de> for AgentCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(AgentCategory::new(&s))
    }
}
