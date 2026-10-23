use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChatHostError {
    #[error("chat session error: {0}")]
    Session(#[from] katana_chat_ui::ChatSessionError),
    #[error("history error: {0}")]
    History(String),
    #[error("output error: {0}")]
    Output(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("unsupported action: {0}")]
    UnsupportedAction(String),
}
