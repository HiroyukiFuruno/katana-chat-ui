use floem::{AnyView, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiSettingsSectionSurface, ChatUiSettingsSurface, ChatUiSurface};

use super::styles;

const PANEL_WIDTH: f64 = 420.0;
const PANEL_RIGHT_INSET: f64 = 24.0;
const PANEL_TOP_INSET: f64 = 72.0;
const PANEL_PADDING: f64 = 18.0;
const PANEL_SHADOW_BLUR: f64 = 24.0;
const PANEL_SHADOW_COLOR: Color = Color::rgba8(15, 23, 42, 35);
const SETTINGS_ITEM_GAP: f64 = 4.0;
const SETTINGS_SECTION_GAP: f64 = 6.0;

pub(super) struct FloemSettingsSurface;

impl FloemSettingsSurface {
    pub(super) fn render(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
        dyn_container(
            move || surface.get().settings,
            |settings| {
                if !settings.visible {
                    return empty().into_any();
                }
                settings_panel(settings)
            },
        )
    }
}

fn settings_panel(settings: ChatUiSettingsSurface) -> AnyView {
    container(settings_content(settings))
        .style(|style| {
            style
                .absolute()
                .inset_right(PANEL_RIGHT_INSET)
                .inset_top(PANEL_TOP_INSET)
                .width(PANEL_WIDTH)
                .padding(PANEL_PADDING)
                .gap(styles::PANEL_GAP)
                .border(1.0)
                .border_color(styles::COLOR_BORDER)
                .border_radius(styles::BUBBLE_RADIUS)
                .background(Color::WHITE)
                .box_shadow_blur(PANEL_SHADOW_BLUR)
                .box_shadow_color(PANEL_SHADOW_COLOR)
                .flex_col()
        })
        .into_any()
}

fn settings_content(settings: ChatUiSettingsSurface) -> impl IntoView {
    v_stack((
        settings_title(),
        settings_reference(settings.reference_path),
        section_stack(settings.sections),
    ))
}

fn settings_title() -> impl IntoView {
    text("Settings").style(|style| {
        style
            .font_size(styles::FONT_BODY)
            .color(styles::COLOR_TEXT)
            .font_bold()
    })
}

fn settings_reference(reference_path: String) -> impl IntoView {
    text(reference_path).style(|style| {
        style
            .font_size(styles::FONT_META)
            .color(styles::COLOR_MUTED)
    })
}

fn section_stack(sections: Vec<ChatUiSettingsSectionSurface>) -> impl IntoView {
    dyn_stack(
        move || sections.clone(),
        |section| section.id.clone(),
        settings_section,
    )
    .style(|style| style.gap(styles::PANEL_GAP).flex_col())
}

fn settings_section(section: ChatUiSettingsSectionSurface) -> AnyView {
    v_stack((
        text(section.label)
            .style(|style| style.font_size(styles::FONT_BODY).color(styles::COLOR_TEXT)),
        dyn_stack(
            move || section.items.clone(),
            |item| item.id.clone(),
            |item| text(format!("{}: {}", item.label, item.value)),
        )
        .style(|style| {
            style
                .gap(SETTINGS_ITEM_GAP)
                .font_size(styles::FONT_META)
                .color(styles::COLOR_MUTED)
                .flex_col()
        }),
    ))
    .style(|style| style.gap(SETTINGS_SECTION_GAP).flex_col())
    .into_any()
}

#[cfg(test)]
mod tests {
    #[test]
    fn settings_surface_is_right_top_overlay() {
        let source = include_str!("settings.rs");

        assert!(source.contains(".inset_right(PANEL_RIGHT_INSET)"));
        assert!(source.contains(".inset_top(PANEL_TOP_INSET)"));
        assert!(source.contains("settings.reference_path"));
    }
}
