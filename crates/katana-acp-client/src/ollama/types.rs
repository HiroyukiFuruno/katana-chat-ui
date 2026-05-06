use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(super) struct OllamaChatRequest {
    pub(super) model: String,
    pub(super) messages: Vec<OllamaMessage>,
    pub(super) stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) think: Option<OllamaThink>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OllamaMessage {
    pub(super) role: String,
    pub(super) content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) thinking: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(super) enum OllamaThink {
    Enabled(bool),
    Effort(String),
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaChatResponse {
    pub(super) message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaChatStreamResponse {
    pub(super) message: Option<OllamaMessage>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaTagsResponse {
    pub(super) models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OllamaModel {
    pub(super) name: String,
}
