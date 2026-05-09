use crate::{composer::EguiComposerView, message::EguiMessageListView};
use eframe::egui;
use katana_chat_ui::{ChatUiColorSpec, ChatUiLayoutSpec, ChatUiSurface};

mod header;

const LAYOUT: ChatUiLayoutSpec = ChatUiLayoutSpec::DEFAULT;
const COLORS: ChatUiColorSpec = ChatUiColorSpec::DEFAULT;

pub trait EguiChatController {
    fn composer_text_mut(&mut self) -> &mut String;
    fn attach(&mut self);
    fn submit(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;

    fn new_chat(&mut self) {}

    fn history(&mut self) {}

    fn select_vendor(&mut self, _vendor_id: String) {}

    fn select_control(&mut self, _key: String, _value: String) {}
}

pub struct EguiChatView;

impl EguiChatView {
    pub fn render(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        ui.ctx().set_visuals(egui::Visuals::light());
        egui::Frame::new()
            .fill(color(COLORS.panel))
            .inner_margin(LAYOUT.surface_padding)
            .show(ui, |ui| Self::content(ui, surface, controller));
    }

    fn content(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        ui.set_min_size(ui.available_size());
        Self::header(ui, surface, controller);
        ui.add_space(LAYOUT.root_gap);
        EguiMessageListView::render(ui, surface, composer_reserved_height());
        ui.add_space(LAYOUT.root_gap);
        centered_max_width(ui, |ui| {
            EguiComposerView::render(ui, surface, controller);
        });
    }

    fn header(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        header::render(ui, surface, controller);
    }
}

fn centered_max_width(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width().min(LAYOUT.chat_body_max_width);
    let left_margin = ((ui.available_width() - width) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(left_margin);
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            add,
        );
    });
}

pub(crate) fn set_control_combo_visuals(ui: &mut egui::Ui) {
    set_combo_visuals(
        ui,
        color(COLORS.assistant),
        egui::Stroke::new(1.0, color(COLORS.border)),
    );
}

fn set_combo_visuals(ui: &mut egui::Ui, fill: egui::Color32, stroke: egui::Stroke) {
    let widgets = &mut ui.style_mut().visuals.widgets;
    widgets.inactive.weak_bg_fill = fill;
    widgets.inactive.bg_stroke = stroke;
    widgets.hovered.weak_bg_fill = fill;
    widgets.hovered.bg_stroke = stroke;
    widgets.active.weak_bg_fill = fill;
    widgets.active.bg_stroke = stroke;
    widgets.open.weak_bg_fill = fill;
    widgets.open.bg_stroke = stroke;
}

fn composer_reserved_height() -> f32 {
    LAYOUT.composer_input_height + (LAYOUT.composer_padding * 2.0) + 150.0
}

pub(crate) fn color(rgb: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

pub(crate) fn layout() -> ChatUiLayoutSpec {
    LAYOUT
}

pub(crate) fn colors() -> ChatUiColorSpec {
    COLORS
}
