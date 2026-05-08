use crate::{composer::EguiComposerView, message::EguiMessageListView};
use eframe::egui;
use katana_chat_ui::{ChatUiColorSpec, ChatUiLayoutSpec, ChatUiSurface};

const LAYOUT: ChatUiLayoutSpec = ChatUiLayoutSpec::DEFAULT;
const COLORS: ChatUiColorSpec = ChatUiColorSpec::DEFAULT;
const PROVIDER_SELECTOR_CHEVRON: &str = "⌄";

pub trait EguiChatController {
    fn composer_text_mut(&mut self) -> &mut String;
    fn attach(&mut self);
    fn submit(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;

    fn new_chat(&mut self) {}

    fn history(&mut self) {}

    fn settings(&mut self) {}

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
        ui.horizontal(|ui| {
            provider_icon(ui, surface);
            provider_selector(ui, surface, controller);
            ui.label(
                egui::RichText::new(&surface.chrome.title)
                    .color(color(COLORS.text))
                    .size(LAYOUT.toolbar_font_size),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                action_button(ui, "⚙", &surface.chrome.settings.label, || {
                    controller.settings()
                });
                action_button(ui, "↺", &surface.chrome.history.label, || {
                    controller.history()
                });
                action_button(ui, "+", &surface.chrome.new_chat.label, || {
                    controller.new_chat()
                });
            });
        });
    }
}

fn provider_icon(ui: &mut egui::Ui, surface: &ChatUiSurface) {
    ui.label(
        egui::RichText::new(provider_icon_symbol(&surface.chrome.provider_icon.asset_id))
            .color(color(COLORS.text))
            .size(LAYOUT.icon_size),
    )
    .on_hover_text(&surface.vendor_bar.active_vendor_label);
}

fn provider_icon_symbol(asset_id: &str) -> &'static str {
    match asset_id {
        "provider:claude-code" => "✺",
        "provider:codex-cli" => "◎",
        "provider:github-copilot" => "◉",
        "provider:ollama" => "⌁",
        "provider:opencode" => "▣",
        _ => "•",
    }
}

fn centered_max_width(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width().min(LAYOUT.chat_body_max_width);
    let left_margin = ((ui.available_width() - width) / 2.0).max(0.0);
    ui.horizontal(|ui| {
        ui.add_space(left_margin);
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            add,
        );
    });
}

fn provider_selector(
    ui: &mut egui::Ui,
    surface: &ChatUiSurface,
    controller: &mut impl EguiChatController,
) {
    let mut selected = None;
    ui.scope(|ui| {
        set_combo_visuals(ui, egui::Color32::TRANSPARENT, provider_selector_stroke());
        ui.style_mut().spacing.button_padding = egui::vec2(4.0, 2.0);
        let label = format!(
            "{} {PROVIDER_SELECTOR_CHEVRON}",
            surface.vendor_bar.active_vendor_label
        );
        ui.menu_button(label, |ui| {
            for option in &surface.vendor_bar.vendor_options {
                if ui
                    .selectable_label(
                        option.id == surface.vendor_bar.active_vendor_id,
                        &option.label,
                    )
                    .clicked()
                {
                    selected = Some(option.id.clone());
                    ui.close();
                }
            }
        });
    });
    if let Some(selected) = selected {
        controller.select_vendor(selected);
    }
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

fn provider_selector_stroke() -> egui::Stroke {
    egui::Stroke::NONE
}

fn action_button(ui: &mut egui::Ui, label: &str, tooltip: &str, action: impl FnOnce()) {
    if ui
        .add(
            egui::Button::new(egui::RichText::new(label).color(color(COLORS.text)))
                .frame(false)
                .min_size(egui::vec2(28.0, 28.0)),
        )
        .on_hover_text(tooltip)
        .clicked()
    {
        action();
    }
}

fn composer_reserved_height() -> f32 {
    LAYOUT.composer_input_height + (LAYOUT.composer_padding * 2.0) + 54.0
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

#[cfg(test)]
mod tests {
    use eframe::egui;

    #[test]
    fn provider_selector_uses_labeled_unframed_combo() {
        assert_eq!(super::provider_selector_stroke(), egui::Stroke::NONE);
        assert_eq!(super::PROVIDER_SELECTOR_CHEVRON, "⌄");
        assert_eq!(super::provider_icon_symbol("provider:claude-code"), "✺");
    }
}
