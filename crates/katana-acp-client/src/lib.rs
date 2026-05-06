//! katana-acp-client: Agent Client Protocol client scaffolding.
//!
//! ACP (Agent Client Protocol) is the protocol adopted by Zed, VS Code, and
//! JetBrains products to integrate LLM agents from multiple vendors behind a
//! single contract. This crate provides a vendor-neutral client API so host
//! applications can talk to any ACP-compatible agent or supported direct
//! provider without per-vendor UI code.
//!
//! Status: scaffolding. Neutral interface and Ollama provider are added
//! during the v0.0.1 change.

pub mod ollama;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcpContentBlock {
    Text(AcpTextContent),
    Image(AcpImageContent),
    EmbeddedResource(AcpEmbeddedResource),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpTextContent {
    pub text: String,
}

impl AcpTextContent {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpImageContent {
    pub mime_type: String,
    pub data_ref: String,
}

impl AcpImageContent {
    pub fn new(mime_type: impl Into<String>, data_ref: impl Into<String>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data_ref: data_ref.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpEmbeddedResource {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
}

impl AcpEmbeddedResource {
    pub fn new(
        uri: impl Into<String>,
        mime_type: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            mime_type: mime_type.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContext {
    pub uri: String,
    pub content: String,
    pub cursor_offset: usize,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiIntent {
    Modify,
    Create,
    Autofix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub intent: AiIntent,
    pub context: DocumentContext,
    pub prompt: String,
    pub history: Vec<ChatTurn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiStreamEvent {
    Content(String),
    Thinking(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapability {
    pub id: String,
}

#[derive(Debug, Error)]
pub enum AcpError {
    #[error("ACP transport not implemented")]
    NotImplemented,
    #[error("transport error: {0}")]
    Transport(String),
    #[error("protocol error: {0}")]
    Protocol(String),
}

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    async fn is_available(&self) -> bool;
    fn capabilities(&self) -> Vec<AiCapability>;
    async fn list_models(&self) -> Result<Vec<AiModel>, AcpError>;
    async fn execute(&self, request: &AiRequest) -> Result<AiResponse, AcpError>;
}

#[cfg(test)]
mod tests {
    use super::{AiIntent, AiRequest, ChatRole, ChatTurn, DocumentContext};

    #[test]
    fn test_ai_request_serde() -> Result<(), serde_json::Error> {
        let req = AiRequest {
            intent: AiIntent::Modify,
            context: DocumentContext {
                uri: "file:///test.rs".to_string(),
                content: "fn main() {}".to_string(),
                cursor_offset: 0,
                diagnostics: vec![],
            },
            prompt: "Refactor this".to_string(),
            history: vec![ChatTurn {
                role: ChatRole::User,
                content: "Hello".to_string(),
            }],
        };

        let json = serde_json::to_string(&req)?;
        let de: AiRequest = serde_json::from_str(&json)?;
        assert_eq!(de.intent, AiIntent::Modify);
        assert_eq!(de.prompt, "Refactor this");
        Ok(())
    }
}
