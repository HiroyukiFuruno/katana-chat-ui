use crate::view::{color, colors, layout};
use eframe::egui;
use katana_chat_ui::{ChatUiMessageAlignment, ChatUiMessageSurface, ChatUiSurface, MarkdownBlock};

pub struct EguiMessageListView;

impl EguiMessageListView {
    pub fn render(ui: &mut egui::Ui, surface: &ChatUiSurface, reserved_height: f32) {
        let height = (ui.available_height() - reserved_height).max(0.0);
        egui::ScrollArea::vertical()
            .max_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.set_width(ui.available_width().min(layout().chat_body_max_width));
                    for message in &surface.message_list.messages {
                        message_row(ui, message);
                    }
                });
            });
    }
}

fn message_row(ui: &mut egui::Ui, message: &ChatUiMessageSurface) {
    let trailing = message.alignment == ChatUiMessageAlignment::Trailing;
    let row_layout = if trailing {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    let row_width = ui.available_width();
    ui.allocate_ui_with_layout(egui::vec2(row_width, 0.0), row_layout, |ui| {
        ui.set_width(row_width);
        message_bubble(ui, message, trailing);
    });
    ui.add_space(layout().message_gap);
}

fn message_bubble(ui: &mut egui::Ui, message: &ChatUiMessageSurface, trailing: bool) {
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
            if trailing {
                ui.set_max_width(layout.user_bubble_max_width);
            } else {
                let width = ui.available_width();
                ui.set_min_width(width);
                ui.set_width(width);
            }
            message_content(ui, message);
        });
}

fn message_content(ui: &mut egui::Ui, message: &ChatUiMessageSurface) {
    if let Some(thinking) = &message.thinking {
        ui.label(&thinking.label);
        for entry in &thinking.entries {
            ui.label(entry);
        }
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
        MarkdownBlock::Heading(heading) => {
            ui.heading(heading.text.plain_text());
        }
        MarkdownBlock::Paragraph(text) | MarkdownBlock::BlockQuote(text) => {
            ui.label(text.plain_text());
        }
        MarkdownBlock::CodeBlock(code) => {
            ui.monospace(&code.code);
        }
        MarkdownBlock::List(list) => {
            for item in &list.items {
                ui.label(format!("- {}", item.plain_text()));
            }
        }
        MarkdownBlock::Table(table) => {
            ui.label(
                table
                    .headers
                    .iter()
                    .map(katana_chat_ui::TextBlock::plain_text)
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
        }
    }
}
