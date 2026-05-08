use gpui::{Div, div, prelude::*};
use katana_chat_ui::{
    ChatUiMessageAlignment, ChatUiMessageSurface, ChatUiSurface, ChatUiThinkingSurface,
    MessageStatus,
};

use super::styles::GpuiStyles;

pub(super) struct GpuiThreadView;

impl GpuiThreadView {
    pub(super) fn render(surface: &ChatUiSurface) -> Div {
        GpuiStyles::centered(
            div()
                .flex()
                .flex_col()
                .gap(GpuiStyles::px(GpuiStyles::LAYOUT.message_gap))
                .w_full()
                .max_w(GpuiStyles::px(GpuiStyles::LAYOUT.chat_body_max_width))
                .p(GpuiStyles::px(GpuiStyles::LAYOUT.thread_padding))
                .children(surface.message_list.messages.iter().map(Self::message)),
        )
        .flex_1()
    }

    fn message(message: &ChatUiMessageSurface) -> Div {
        let trailing = message.alignment == ChatUiMessageAlignment::Trailing;
        div()
            .flex()
            .w_full()
            .px(GpuiStyles::px(5.0))
            .when(trailing, |view| view.justify_end())
            .when(!trailing, |view| view.justify_start())
            .child(Self::bubble(message, trailing))
    }

    fn bubble(message: &ChatUiMessageSurface, trailing: bool) -> Div {
        let body = Self::visible_body(message);
        let mut bubble = div()
            .rounded_lg()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.border))
            .bg(Self::message_background(trailing))
            .px(GpuiStyles::px(GpuiStyles::LAYOUT.bubble_padding_x))
            .py(GpuiStyles::px(GpuiStyles::LAYOUT.bubble_padding_y))
            .when(!trailing, |view| view.w_full())
            .when(trailing, |view| {
                view.max_w(GpuiStyles::px(GpuiStyles::LAYOUT.user_bubble_max_width))
            });
        if let Some(thinking) = &message.thinking {
            bubble = bubble.child(Self::thinking(thinking));
        }
        if body.is_empty() {
            return bubble;
        }
        bubble.child(body)
    }

    fn thinking(thinking: &ChatUiThinkingSurface) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(GpuiStyles::px(4.0))
            .rounded_md()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.border))
            .px_3()
            .py_2()
            .child(thinking.label.clone())
            .children(thinking.entries.iter().cloned())
    }

    fn visible_body(message: &ChatUiMessageSurface) -> String {
        if !message.body.is_empty() {
            return message.body.clone();
        }
        if message.thinking.is_some() {
            return String::new();
        }
        match &message.status {
            MessageStatus::Sending | MessageStatus::Streaming => message.status_label.clone(),
            MessageStatus::Error(error) => error.clone(),
            MessageStatus::Complete => String::new(),
        }
    }

    fn message_background(trailing: bool) -> gpui::Rgba {
        if trailing {
            return GpuiStyles::color(GpuiStyles::COLORS.user);
        }
        GpuiStyles::color(GpuiStyles::COLORS.assistant)
    }
}

#[cfg(test)]
mod tests {
    use katana_chat_ui::{ChatSession, ChatUiSurface, ThinkingLog};

    #[test]
    fn thinking_message_does_not_fallback_to_status_label()
    -> Result<(), katana_chat_ui::ChatSessionError> {
        let mut session = ChatSession::new();
        session.set_provider_configured("Claude Code");
        session.start_assistant_stream_with_thinking(
            "",
            ThinkingLog::running("Thinking", vec!["入力を確認中".to_string()]),
        )?;
        let surface = ChatUiSurface::from_render_model(&session.render_model());
        let body = surface
            .messages
            .iter()
            .find(|it| it.thinking.is_some())
            .map(super::GpuiThreadView::visible_body);

        assert_eq!(body, Some(String::new()));
        Ok(())
    }
}
