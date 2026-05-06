use katana_chat_ui::{MessageRenderModel, MessageRole, MessageStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageListView {
    pub messages: Vec<MessageRowView>,
}

impl MessageListView {
    pub fn from_model(messages: &[MessageRenderModel]) -> Self {
        Self {
            messages: messages.iter().map(MessageRowView::from_model).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRowView {
    pub role: MessageRole,
    pub status: MessageStatus,
    pub block_count: usize,
    pub attachment_count: usize,
}

impl MessageRowView {
    fn from_model(message: &MessageRenderModel) -> Self {
        Self {
            role: message.role,
            status: message.status.clone(),
            block_count: message.blocks.len(),
            attachment_count: message.attachments.len(),
        }
    }
}
