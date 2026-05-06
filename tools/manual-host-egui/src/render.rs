use crate::composer_render::ManualComposerRenderer;
use crate::output_render::ManualOutputRenderer;
use crate::state::ManualHostState;
use eframe::egui;
use katana_chat_ui::{ChatRenderModel, MarkdownBlock, MessageRenderModel};

pub struct ManualHostRenderer;

impl ManualHostRenderer {
    pub fn render(
        ui: &mut egui::Ui,
        state: &mut ManualHostState,
        last_error: &mut Option<String>,
    ) {
        Self::top_bar(ui, state, last_error);
        ui.separator();
        Self::chat_ui(ui, state, last_error);
    }

    fn top_bar(
        ui: &mut egui::Ui,
        state: &ManualHostState,
        last_error: &Option<String>,
    ) {
        ui.horizontal(|ui| {
            ui.heading("katana-chat-ui egui host");
            ui.label(state.font_status_label());
        });
        ui.horizontal(|ui| {
            ui.label(format!("最後の操作: {}", state.last_action()));
            if let Some(error) = last_error {
                ui.colored_label(egui::Color32::from_rgb(180, 40, 40), error);
            }
        });
    }

    fn chat_ui(ui: &mut egui::Ui, state: &mut ManualHostState, last_error: &mut Option<String>) {
        ui.columns(2, |columns| {
            columns[0].heading("会話");
            Self::message_list(&mut columns[0], &state.render_model());
            ManualOutputRenderer::render(&mut columns[1], state, last_error);
        });
        ui.separator();
        ManualComposerRenderer::render(ui, state, last_error);
    }

    fn message_list(ui: &mut egui::Ui, model: &ChatRenderModel) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for message in &model.messages {
                Self::message(ui, message);
                ui.add_space(8.0);
            }
        });
    }

    fn message(ui: &mut egui::Ui, message: &MessageRenderModel) {
        ui.group(|ui| {
            ui.label(format!("{:?} / {:?}", message.role, message.status));
            for block in &message.blocks {
                Self::markdown_block(ui, block);
            }
            ui.label(format!("添付: {} 件", message.attachments.len()));
        });
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
                ui.label(format!("list: {} 件", list.items.len()));
            }
            MarkdownBlock::Table(table) => {
                ui.label(format!("table: {} rows", table.rows.len()));
            }
        }
    }
}
