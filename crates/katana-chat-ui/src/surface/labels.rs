use super::model::ChatUiMessageAlignment;
use crate::{AgentActivityKind, ChatRenderModel, MessageRole, MessageStatus};

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
                model.texts.processing_status.clone()
            }
            MessageStatus::Complete => String::new(),
            MessageStatus::Error(error) => error.clone(),
        }
    }

    pub(super) fn activity_label(activity: AgentActivityKind, model: &ChatRenderModel) -> String {
        match activity {
            AgentActivityKind::Processing => model.texts.processing_status.clone(),
            AgentActivityKind::Generating => model.texts.generating_status.clone(),
            AgentActivityKind::Editing => model.texts.editing_status.clone(),
            AgentActivityKind::Reading => model.texts.reading_status.clone(),
            AgentActivityKind::Searching => model.texts.searching_status.clone(),
            AgentActivityKind::WebSearching => model.texts.web_searching_status.clone(),
            AgentActivityKind::Executing => model.texts.executing_status.clone(),
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
