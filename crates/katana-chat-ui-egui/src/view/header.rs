use super::{EguiChatController, color, colors, layout, set_combo_visuals};
use eframe::egui;
use katana_chat_ui::ChatUiSurface;

const PROVIDER_SELECTOR_CHEVRON: &str = "v";
const COMPACT_HEADER_WIDTH: f32 = 520.0;

pub(super) fn render(
    ui: &mut egui::Ui,
    surface: &ChatUiSurface,
    controller: &mut impl EguiChatController,
) {
    if ui.available_width() < COMPACT_HEADER_WIDTH {
        compact_header(ui, surface, controller);
        return;
    }
    wide_header(ui, surface, controller);
}

fn compact_header(
    ui: &mut egui::Ui,
    surface: &ChatUiSurface,
    controller: &mut impl EguiChatController,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            provider_icon(ui, surface);
            provider_selector(ui, surface, controller);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                action_button(ui, "↺", &surface.chrome.history.label, || {
                    controller.history()
                });
                action_button(ui, "+", &surface.chrome.new_chat.label, || {
                    controller.new_chat()
                });
            });
        });
        ui.label(
            egui::RichText::new(&surface.chrome.title)
                .color(color(colors().text))
                .size(layout().toolbar_font_size),
        );
    });
}

fn wide_header(
    ui: &mut egui::Ui,
    surface: &ChatUiSurface,
    controller: &mut impl EguiChatController,
) {
    ui.horizontal(|ui| {
        provider_icon(ui, surface);
        provider_selector(ui, surface, controller);
        ui.label(
            egui::RichText::new(&surface.chrome.title)
                .color(color(colors().text))
                .size(layout().toolbar_font_size),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            action_button(ui, "↺", &surface.chrome.history.label, || {
                controller.history()
            });
            action_button(ui, "+", &surface.chrome.new_chat.label, || {
                controller.new_chat()
            });
        });
    });
}

fn provider_icon(ui: &mut egui::Ui, surface: &ChatUiSurface) {
    ui.label(
        egui::RichText::new(provider_icon_symbol(&surface.chrome.provider_icon.asset_id))
            .color(color(colors().text))
            .size(layout().icon_size),
    )
    .on_hover_text(&surface.vendor_bar.active_vendor_label);
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

fn provider_icon_symbol(asset_id: &str) -> &'static str {
    match asset_id {
        "provider:claude-code" => "CC",
        "provider:codex-cli" => "CX",
        "provider:github-copilot" => "GH",
        "provider:ollama" => "OL",
        "provider:opencode" => "OC",
        _ => "*",
    }
}

fn provider_selector_stroke() -> egui::Stroke {
    egui::Stroke::NONE
}

fn action_button(ui: &mut egui::Ui, label: &str, tooltip: &str, action: impl FnOnce()) {
    if ui
        .add(
            egui::Button::new(egui::RichText::new(label).color(color(colors().text)))
                .frame(false)
                .min_size(egui::vec2(28.0, 28.0)),
        )
        .on_hover_text(tooltip)
        .clicked()
    {
        action();
    }
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    #[test]
    fn provider_selector_uses_labeled_unframed_combo() {
        assert_eq!(super::provider_selector_stroke(), egui::Stroke::NONE);
        assert_eq!(super::PROVIDER_SELECTOR_CHEVRON, "v");
        assert_eq!(super::COMPACT_HEADER_WIDTH, 520.0);
        assert_eq!(super::provider_icon_symbol("provider:claude-code"), "CC");
    }

    #[test]
    fn provider_icon_symbols_are_ascii_safe_for_headless_baselines() {
        for provider in [
            "provider:claude-code",
            "provider:codex-cli",
            "provider:github-copilot",
            "provider:ollama",
            "provider:opencode",
            "provider:unknown",
        ] {
            assert!(super::provider_icon_symbol(provider).is_ascii());
        }
    }
}
