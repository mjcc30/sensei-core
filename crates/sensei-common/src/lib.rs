use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Health {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AskRequest {
    pub prompt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AskResponse {
    pub content: String,
}
