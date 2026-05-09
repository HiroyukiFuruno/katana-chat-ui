use crate::view::{color, colors, layout};
use eframe::egui;
use katana_chat_ui::{
    ChatUiMessageAlignment, ChatUiMessageSurface, ChatUiSurface, CodeBlock, HeadingBlock, ListItem,
    ListKind, MarkdownBlock, TableBlock, TextBlock,
};

pub struct EguiMessageListView;

impl EguiMessageListView {
    pub fn render(ui: &mut egui::Ui, surface: &ChatUiSurface, reserved_height: f32) {
        let height = (ui.available_height() - reserved_height).max(0.0);
        let outer_width = ui.available_width();
        let body_width = body_width(outer_width);
        let left_margin = ((outer_width - body_width) / 2.0).max(0.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.add_space(left_margin);
            ui.allocate_ui_with_layout(
                egui::vec2(body_width, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(body_width);
                            for message in &surface.message_list.messages {
                                message_row(ui, message, body_width);
                            }
                        });
                },
            );
        });
    }
}

fn body_width(outer_width: f32) -> f32 {
    outer_width.min(layout().chat_body_max_width).max(0.0)
}

fn message_row(ui: &mut egui::Ui, message: &ChatUiMessageSurface, row_width: f32) {
    let trailing = message.alignment == ChatUiMessageAlignment::Trailing;
    let bubble_width = message_outer_width(message, trailing, row_width, layout());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if trailing {
            ui.add_space((row_width - bubble_width).max(0.0));
        }
        message_bubble(ui, message, trailing, bubble_width);
    });
    ui.add_space(layout().message_gap);
}

fn message_bubble(
    ui: &mut egui::Ui,
    message: &ChatUiMessageSurface,
    trailing: bool,
    outer_width: f32,
) {
    let layout = layout();
    let colors = colors();
    let background = if trailing {
        colors.user
    } else {
        colors.assistant
    };
    egui::Frame::new()
        .fill(color(background))
        .stroke(egui::Stroke::new(1.0, color(colors.border)))
        .corner_radius(layout.bubble_radius)
        .inner_margin(egui::Margin::symmetric(
            layout.bubble_padding_x as i8,
            layout.bubble_padding_y as i8,
        ))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                let width = content_width(outer_width, layout);
                ui.set_min_width(width);
                ui.set_width(width);
                message_content(ui, message);
            });
        });
}

fn message_outer_width(
    message: &ChatUiMessageSurface,
    trailing: bool,
    row_width: f32,
    layout: katana_chat_ui::ChatUiLayoutSpec,
) -> f32 {
    if message.body.trim().is_empty() && (message.activity.is_some() || message.thinking.is_some())
    {
        return state_outer_width(message, row_width, layout);
    }
    if trailing {
        return user_outer_width(message, row_width, layout);
    }
    assistant_outer_width(row_width, layout)
}

fn state_outer_width(
    message: &ChatUiMessageSurface,
    row_width: f32,
    layout: katana_chat_ui::ChatUiLayoutSpec,
) -> f32 {
    (state_content_width(message, row_width, layout) + (layout.bubble_padding_x * 2.0))
        .min(row_width)
        .max(0.0)
}

fn assistant_outer_width(row_width: f32, layout: katana_chat_ui::ChatUiLayoutSpec) -> f32 {
    (row_width * (layout.agent_bubble_width_percent / 100.0)).max(0.0)
}

fn content_width(outer_width: f32, layout: katana_chat_ui::ChatUiLayoutSpec) -> f32 {
    (outer_width - (layout.bubble_padding_x * 2.0)).max(0.0)
}

fn user_outer_width(
    message: &ChatUiMessageSurface,
    row_width: f32,
    layout: katana_chat_ui::ChatUiLayoutSpec,
) -> f32 {
    (user_content_width(message, row_width, layout) + (layout.bubble_padding_x * 2.0))
        .min(row_width)
        .max(0.0)
}

fn user_content_max_width(row_width: f32, layout: katana_chat_ui::ChatUiLayoutSpec) -> f32 {
    (row_width.min(layout.user_bubble_max_width) - (layout.bubble_padding_x * 2.0)).max(0.0)
}

fn user_content_width(
    message: &ChatUiMessageSurface,
    row_width: f32,
    layout: katana_chat_ui::ChatUiLayoutSpec,
) -> f32 {
    let estimated_text_width = longest_line_chars(&message.body) as f32 * layout.font_body + 16.0;
    estimated_text_width
        .max(layout.font_body * 2.0)
        .min(user_content_max_width(row_width, layout))
}

fn state_content_width(
    message: &ChatUiMessageSurface,
    row_width: f32,
    layout: katana_chat_ui::ChatUiLayoutSpec,
) -> f32 {
    let label = if let Some(activity) = &message.activity {
        activity.label.as_str()
    } else if let Some(thinking) = &message.thinking {
        thinking.label.as_str()
    } else {
        ""
    };
    let estimated_text_width = longest_line_chars(label) as f32 * layout.font_body + 16.0;
    estimated_text_width
        .max(layout.font_body * 4.0)
        .min(user_content_max_width(row_width, layout))
}

fn longest_line_chars(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .fold(0, usize::max)
}

fn message_content(ui: &mut egui::Ui, message: &ChatUiMessageSurface) {
    if let Some(thinking) = &message.thinking {
        wrapped_label(ui, &thinking.label);
        for entry in &thinking.entries {
            wrapped_label(ui, entry);
        }
    }
    if message.body.trim().is_empty()
        && let Some(activity) = &message.activity
    {
        wrapped_label(ui, &activity.label);
    }
    if message.body.trim().is_empty() {
        return;
    }
    for block in &message.blocks {
        markdown_block(ui, block);
    }
}

fn markdown_block(ui: &mut egui::Ui, block: &MarkdownBlock) {
    match block {
        MarkdownBlock::Heading(heading) => markdown_heading(ui, heading),
        MarkdownBlock::Paragraph(text) | MarkdownBlock::BlockQuote(text) => {
            markdown_text(ui, text);
        }
        MarkdownBlock::CodeBlock(code) => markdown_code(ui, code),
        MarkdownBlock::List(list) => markdown_list(ui, &list.kind, &list.items),
        MarkdownBlock::Table(table) => markdown_table(ui, table),
    }
}

fn markdown_heading(ui: &mut egui::Ui, heading: &HeadingBlock) {
    wrapped_rich_label(
        ui,
        egui::RichText::new(heading.text.plain_text())
            .heading()
            .strong(),
    );
}

fn markdown_text(ui: &mut egui::Ui, text: &TextBlock) {
    wrapped_label(ui, &text.plain_text());
}

fn markdown_code(ui: &mut egui::Ui, code: &CodeBlock) {
    wrapped_rich_label(ui, egui::RichText::new(&code.code).monospace());
}

fn markdown_list(ui: &mut egui::Ui, kind: &ListKind, items: &[ListItem]) {
    for (index, item) in items.iter().enumerate() {
        let marker = list_marker(kind, index, item.checked);
        wrapped_label(ui, &format!("{} {}", marker, item.plain_text()));
    }
}

fn markdown_table(ui: &mut egui::Ui, table: &TableBlock) {
    if !table.headers.is_empty() {
        wrapped_label(ui, &table_row_text(&table.headers));
    }
    for row in &table.rows {
        wrapped_label(ui, &table_row_text(row));
    }
}

fn wrapped_label(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(text).wrap());
}

fn wrapped_rich_label(ui: &mut egui::Ui, text: egui::RichText) {
    ui.add(egui::Label::new(text).wrap());
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

fn table_row_text(row: &[katana_chat_ui::TextBlock]) -> String {
    row.iter()
        .map(katana_chat_ui::TextBlock::plain_text)
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests {
    use super::{
        assistant_outer_width, body_width, list_marker, longest_line_chars, state_outer_width,
        table_row_text, user_outer_width,
    };
    use katana_chat_ui::{
        AgentActivityKind, Attachment, ChatUiActivitySurface, ChatUiLayoutSpec,
        ChatUiMessageAlignment, ChatUiMessageSurface, ListKind, MarkdownBlock, MessageRole,
        MessageStatus, TextBlock,
    };

    #[test]
    fn list_marker_preserves_task_and_ordered_state() {
        assert_eq!(list_marker(&ListKind::Unordered, 0, Some(true)), "[x]");
        assert_eq!(list_marker(&ListKind::Unordered, 0, Some(false)), "[ ]");
        assert_eq!(list_marker(&ListKind::Ordered { start: 3 }, 2, None), "5.");
    }

    #[test]
    fn table_row_text_keeps_body_rows_visible() {
        let row = vec![
            TextBlock::from_text("priority"),
            TextBlock::from_text("scope"),
        ];

        assert_eq!(table_row_text(&row), "priority | scope");
    }

    #[test]
    fn assistant_outer_width_keeps_bubble_inside_row() {
        let layout = ChatUiLayoutSpec::DEFAULT;

        assert_eq!(assistant_outer_width(400.0, layout), 400.0);
    }

    #[test]
    fn body_width_is_capped_by_available_width() {
        assert_eq!(body_width(320.0), 320.0);
    }

    #[test]
    fn user_content_width_does_not_escape_narrow_rows() {
        let layout = ChatUiLayoutSpec::DEFAULT;

        assert_eq!(super::user_content_max_width(320.0, layout), 288.0);
    }

    #[test]
    fn user_content_width_uses_message_length_before_row_cap() {
        let layout = ChatUiLayoutSpec::DEFAULT;
        let message = ChatUiMessageSurface {
            id: 1,
            role: MessageRole::User,
            role_label: "User".to_string(),
            status: MessageStatus::Complete,
            status_label: "Complete".to_string(),
            activity: None,
            alignment: ChatUiMessageAlignment::Trailing,
            body: "short".to_string(),
            blocks: vec![MarkdownBlock::Paragraph(TextBlock::from_text("short"))],
            thinking: None,
            outputs: Vec::new(),
            attachments: Vec::<Attachment>::new(),
        };

        assert_eq!(super::user_content_width(&message, 320.0, layout), 91.0);
    }

    #[test]
    fn user_outer_width_adds_padding_without_forcing_full_row() {
        let layout = ChatUiLayoutSpec::DEFAULT;
        let message = ChatUiMessageSurface {
            id: 1,
            role: MessageRole::User,
            role_label: "User".to_string(),
            status: MessageStatus::Complete,
            status_label: "Complete".to_string(),
            activity: None,
            alignment: ChatUiMessageAlignment::Trailing,
            body: "short".to_string(),
            blocks: vec![MarkdownBlock::Paragraph(TextBlock::from_text("short"))],
            thinking: None,
            outputs: Vec::new(),
            attachments: Vec::<Attachment>::new(),
        };

        assert_eq!(user_outer_width(&message, 320.0, layout), 123.0);
    }

    #[test]
    fn state_outer_width_uses_content_size_without_forcing_full_row() {
        let layout = ChatUiLayoutSpec::DEFAULT;
        let message = ChatUiMessageSurface {
            id: 1,
            role: MessageRole::Assistant,
            role_label: "Assistant".to_string(),
            status: MessageStatus::Streaming,
            status_label: "Processing".to_string(),
            activity: Some(ChatUiActivitySurface {
                kind: AgentActivityKind::Processing,
                label: "Processing".to_string(),
            }),
            alignment: ChatUiMessageAlignment::Leading,
            body: String::new(),
            blocks: Vec::new(),
            thinking: None,
            outputs: Vec::new(),
            attachments: Vec::<Attachment>::new(),
        };

        assert_eq!(state_outer_width(&message, 320.0, layout), 198.0);
    }

    #[test]
    fn longest_line_chars_uses_the_longest_line() {
        assert_eq!(longest_line_chars("a\nabcd\nab"), 4);
    }
}
