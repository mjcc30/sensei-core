use serde::{Deserialize, Serialize};

/// Represents a health check response.
///
/// # Examples
///
/// ```
/// use sensei_common::Health;
/// use serde_json::json;
///
/// let health = Health { status: "ok".to_string() };
/// let json = serde_json::to_string(&health).unwrap();
/// assert_eq!(json, r#"{"status":"ok"}"#);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct Health {
    pub status: String,
}

/// Represents a user request to the AI.
///
/// # Examples
///
/// ```
/// use sensei_common::AskRequest;
///
/// let req = AskRequest { prompt: "Hello".to_string() };
/// assert_eq!(req.prompt, "Hello");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct AskRequest {
    pub prompt: String,
}

/// Represents the AI's response.
#[derive(Serialize, Deserialize, Debug)]
pub struct AskResponse {
    pub content: String,
}
