//! Framework 非依存の手動 host 用状態・永続化・出力実行境界。

mod actions;
mod error;
mod history;
mod managed_host;
mod markdown_output;
mod ollama_provider;
mod output;
mod provider;
mod provider_integration;
mod provider_manager;
mod provider_profiles;
mod state;
mod surface;

#[cfg(test)]
mod control_tests;
#[cfg(test)]
mod output_failure_tests;
#[cfg(test)]
mod tests;

pub use error::ChatHostError;
pub(crate) use history::PreparedHistorySave;
pub use history::{ChatHistoryIssue, ChatHistoryListing, ChatHistoryRecord, ChatHistoryStore};
pub use katana_acp_client::protocol::AcpPermissionDecision;
pub use managed_host::{
    ManagedChatHost, ManagedHostCommand, ManagedHostTransportError, ManagedHostUpdate,
};
pub use markdown_output::{MarkdownOutputError, from_ai_response};
pub use ollama_provider::{KatanAgentOllamaEvent, KatanAgentOllamaExecutor, KatanAgentOllamaModel};
pub use output::ChatOutputExecutor;
pub use provider::{
    AcpProviderExecutor, AcpProviderModel, AcpProviderProfile, AcpProviderProjection,
};
pub use provider_integration::AcpHostBridge;
pub use provider_manager::{ProviderManager, ProviderPollEvent, ProviderPollKind};
pub use provider_profiles::acp_provider_profile;
pub use state::{ChatHostEffect, ChatHostState, ChatProviderPoll};
