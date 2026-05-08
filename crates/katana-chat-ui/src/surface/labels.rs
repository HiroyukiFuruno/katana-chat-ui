use super::model::ChatUiMessageAlignment;
use crate::{ChatRenderModel, MessageRole, MessageStatus};

pub(super) struct MessageLabelBuilder;

impl MessageLabelBuilder {
    pub(super) fn role_label(role: MessageRole, model: &ChatRenderModel) -> String {
        match role {
            MessageRole::User => model.texts.user_role.clone(),
            MessageRole::Assistant => model.texts.assistant_role.clone(),
            MessageRole::System => model.texts.system_role.clone(),
            MessageRole::Tool => model.texts.tool_role.clone(),
        }
    }

    pub(super) fn status_label(status: &MessageStatus, model: &ChatRenderModel) -> String {
        match status {
            MessageStatus::Sending | MessageStatus::Streaming => {
                model.texts.thinking_selector.clone()
            }
            MessageStatus::Complete => String::new(),
            MessageStatus::Error(error) => error.clone(),
        }
    }

    pub(super) fn alignment(role: MessageRole) -> ChatUiMessageAlignment {
        match role {
            MessageRole::User => ChatUiMessageAlignment::Trailing,
            MessageRole::Assistant | MessageRole::System | MessageRole::Tool => {
                ChatUiMessageAlignment::Leading
            }
        }
    }
}
