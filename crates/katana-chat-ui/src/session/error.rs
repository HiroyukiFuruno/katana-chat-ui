use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChatSessionError {
    #[error("draft is empty")]
    DraftEmpty,
    #[error("provider is not configured")]
    ProviderMissing,
    #[error("streaming assistant message was not found")]
    StreamingMessageNotFound,
    #[error("message was not found")]
    MessageNotFound,
    #[error("output was not found")]
    OutputNotFound,
}
