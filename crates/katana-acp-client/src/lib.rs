//! katana-acp-client: Agent Client Protocol client scaffolding.
//!
//! ACP (Agent Client Protocol) is the protocol adopted by Zed, VS Code, and
//! JetBrains products to integrate LLM agents from multiple vendors behind a
//! single contract. This crate provides a vendor-neutral client API so KatanA
//! and other consumers can talk to any ACP-compatible agent (Ollama wrapper,
//! Vertex AI, Bedrock, OpenAI-compatible, etc.) without per-vendor UI code.
//!
//! Status: scaffolding. Wire protocol, transport (stdio / WebSocket), and
//! capability negotiation are added during the v0.22.14 change.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub display_name: String,
    pub vendor: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
    System,
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
pub trait AcpClient: Send + Sync {
    async fn agent_info(&self) -> Result<AgentInfo, AcpError>;
    async fn send_turn(&self, history: &[ChatTurn], prompt: &str) -> Result<ChatTurn, AcpError>;
}
