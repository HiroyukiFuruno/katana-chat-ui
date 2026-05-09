mod controls;
mod icons;

use self::controls::EguiComposerControls;
use crate::view::{EguiChatController, color, colors, layout};
use eframe::egui;
use katana_chat_ui::ChatUiSurface;

pub struct EguiComposerView;

pub(super) const CONTROL_ROW_GAP: f32 = 10.0;
pub(super) const ICON_BUTTON_SIZE: f32 = 38.0;

impl EguiComposerView {
    pub fn render(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let layout = layout();
        let colors = colors();
        egui::Frame::new()
            .fill(color(colors.assistant))
            .stroke(egui::Stroke::new(1.0, color(colors.border)))
            .corner_radius(layout.bubble_radius)
            .inner_margin(layout.composer_padding)
            .show(ui, |ui| Self::content(ui, surface, controller));
    }

    fn content(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let layout = layout();
        let content_width = (ui.available_width() - (layout.composer_padding * 2.0))
            .max(0.0)
            .min(layout.chat_body_max_width);
        ui.set_width(content_width);
        ui.add_sized(
            [content_width, layout.composer_input_height],
            egui::TextEdit::multiline(controller.composer_text_mut())
                .frame(egui::Frame::NONE)
                .hint_text(&surface.composer.placeholder),
        );
        ui.add_space(CONTROL_ROW_GAP);
        Self::controls(ui, surface, controller);
    }

    fn controls(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        EguiComposerControls::render(ui, surface, controller);
    }
}
