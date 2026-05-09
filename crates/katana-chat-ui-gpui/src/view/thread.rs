use gpui::{Div, div, prelude::*};
use katana_chat_ui::{
    ChatUiMessageAlignment, ChatUiMessageSurface, ChatUiSurface, ChatUiThinkingSurface, ListKind,
    MarkdownBlock, MessageStatus, TextBlock,
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
        let state_sized = body.is_empty()
            || (message.body.trim().is_empty()
                && (message.activity.is_some() || message.thinking.is_some()));
        let mut bubble = div()
            .rounded_lg()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.border))
            .bg(Self::message_background(trailing))
            .px(GpuiStyles::px(GpuiStyles::LAYOUT.bubble_padding_x))
            .py(GpuiStyles::px(GpuiStyles::LAYOUT.bubble_padding_y))
            .when(!trailing && !state_sized, |view| view.w_full())
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
        let structured_body = Self::structured_body(message);
        if !structured_body.is_empty() {
            return structured_body;
        }
        if let Some(activity) = &message.activity {
            return activity.label.clone();
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

    fn structured_body(message: &ChatUiMessageSurface) -> String {
        if message.blocks.is_empty() {
            return message.body.clone();
        }
        message
            .blocks
            .iter()
            .map(block_text)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn message_background(trailing: bool) -> gpui::Rgba {
        if trailing {
            return GpuiStyles::color(GpuiStyles::COLORS.user);
        }
        GpuiStyles::color(GpuiStyles::COLORS.assistant)
    }
}

fn block_text(block: &MarkdownBlock) -> String {
    match block {
        MarkdownBlock::Heading(heading) => heading.text.plain_text(),
        MarkdownBlock::Paragraph(text) | MarkdownBlock::BlockQuote(text) => text.plain_text(),
        MarkdownBlock::CodeBlock(code) => code.code.clone(),
        MarkdownBlock::List(list) => list
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                format!(
                    "{} {}",
                    list_marker(&list.kind, index, item.checked),
                    item.content.plain_text()
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::Table(table) => table_text(&table.headers, &table.rows),
    }
}

fn table_text(headers: &[TextBlock], rows: &[Vec<TextBlock>]) -> String {
    let mut lines = Vec::new();
    if !headers.is_empty() {
        lines.push(row_text(headers));
    }
    lines.extend(rows.iter().map(|row| row_text(row)));
    lines.join("\n")
}

fn row_text(row: &[TextBlock]) -> String {
    row.iter()
        .map(TextBlock::plain_text)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn list_marker(kind: &ListKind, index: usize, checked: Option<bool>) -> String {
    match checked {
        Some(true) => "[x]".to_string(),
        Some(false) => "[ ]".to_string(),
        None => match kind {
            ListKind::Ordered { start } => format!("{}.", start + index as u64),
            ListKind::Unordered => "-".to_string(),
        },
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

    #[test]
    fn visible_body_uses_structured_markdown_blocks() -> Result<(), katana_chat_ui::ChatSessionError>
    {
        let mut session = ChatSession::new();
        session.set_provider_configured("Claude Code");
        session.draft_mut().set_text("サンプルを出して");
        session.submit_draft()?;
        session.start_assistant_stream(
            "# Title\n\n1. first\n2. second\n\n| A | B |\n|---|---|\n| c | d |",
        )?;
        session.finish_assistant_message()?;
        let surface = ChatUiSurface::from_render_model(&session.render_model());
        let body = surface
            .message_list
            .messages
            .last()
            .map(super::GpuiThreadView::visible_body)
            .unwrap_or_default();

        assert!(body.contains("Title"));
        assert!(body.contains("1. first"));
        assert!(body.contains("A | B"));
        assert!(body.contains("c | d"));
        Ok(())
    }
}
