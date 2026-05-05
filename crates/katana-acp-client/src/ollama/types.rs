use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(super) struct OllamaChatRequest {
    pub(super) model: String,
    pub(super) messages: Vec<OllamaMessage>,
    pub(super) stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OllamaMessage {
    pub(super) role: String,
    pub(super) content: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaChatResponse {
    pub(super) message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaTagsResponse {
    pub(super) models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaModel {
    pub(super) name: String,
}
