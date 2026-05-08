#[cfg(test)]
use super::styles;
use katana_chat_ui::{ChatUiMessageAlignment, ChatUiMessageSurface, MessageStatus};

pub(super) struct ThreadMessagePresenter;

#[cfg(test)]
#[derive(Debug, PartialEq)]
pub(super) struct BubbleVerticalMetrics {
    pub(super) padding_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MessageBubbleLayout {
    AgentFixed,
    ContentSized,
}

impl ThreadMessagePresenter {
    pub(super) fn message_key(message: &ChatUiMessageSurface) -> (u64, &'static str, usize, usize) {
        (
            message.id,
            Self::status_key(&message.status),
            message.body.len(),
            message.outputs.len(),
        )
    }

    pub(super) fn visible_body(message: &ChatUiMessageSurface) -> String {
        if !message.body.is_empty() {
            return message.body.clone();
        }
        if message.thinking.is_none() {
            return String::new();
        }
        match &message.status {
            MessageStatus::Sending | MessageStatus::Streaming => message.status_label.clone(),
            MessageStatus::Error(error) => error.clone(),
            MessageStatus::Complete => String::new(),
        }
    }

    pub(super) fn bubble_body(message: &ChatUiMessageSurface) -> String {
        Self::visible_body(message)
    }

    pub(super) fn bubble_layout(message: &ChatUiMessageSurface) -> MessageBubbleLayout {
        match message.alignment {
            ChatUiMessageAlignment::Leading => MessageBubbleLayout::AgentFixed,
            ChatUiMessageAlignment::Trailing => MessageBubbleLayout::ContentSized,
        }
    }

    #[cfg(test)]
    pub(super) fn agent_bubble_width_percent() -> f64 {
        styles::AGENT_BUBBLE_WIDTH_PERCENT
    }

    #[cfg(test)]
    pub(super) fn chat_body_max_width() -> f64 {
        styles::CHAT_BODY_MAX_WIDTH
    }

    #[cfg(test)]
    pub(super) fn bubble_vertical_metrics() -> BubbleVerticalMetrics {
        BubbleVerticalMetrics {
            padding_y: styles::BUBBLE_PADDING_Y,
        }
    }

    pub(super) fn is_waiting_indicator(message: &ChatUiMessageSurface) -> bool {
        matches!(
            message.status,
            MessageStatus::Sending | MessageStatus::Streaming
        ) && Self::visible_body(message).trim() == message.status_label
    }

    fn status_key(status: &MessageStatus) -> &'static str {
        match status {
            MessageStatus::Sending => "sending",
            MessageStatus::Streaming => "streaming",
            MessageStatus::Complete => "complete",
            MessageStatus::Error(_) => "error",
        }
    }
}
