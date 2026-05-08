use super::styles;
use floem::{peniko::Color, prelude::*};
use katana_chat_ui::ChatUiActionButtonSurface;

const BUTTON_SIZE: f64 = 38.0;

pub struct FloemActionIconButton;

impl FloemActionIconButton {
    pub fn render<Action>(action: ChatUiActionButtonSurface, on_click: Action) -> impl IntoView
    where
        Action: Fn() + Clone + 'static,
    {
        let enabled = action.enabled;
        Self::render_with_enabled(action, move || enabled, on_click)
    }

    pub fn render_with_enabled<Action, Enabled>(
        action: ChatUiActionButtonSurface,
        is_enabled: Enabled,
        on_click: Action,
    ) -> impl IntoView
    where
        Action: Fn() + Clone + 'static,
        Enabled: Fn() -> bool + Clone + 'static,
    {
        let label = action.label.clone();
        let icon_svg = action.icon.svg;
        let icon_enabled = is_enabled.clone();
        container(svg(icon_svg).style(move |style| {
            style
                .size(styles::ICON_SIZE, styles::ICON_SIZE)
                .color(if icon_enabled() {
                    styles::COLOR_TEXT
                } else {
                    styles::COLOR_MUTED
                })
        }))
        .on_click_stop(move |_| {
            if is_enabled() {
                on_click();
            }
        })
        .tooltip(move || tooltip_label(label.clone()))
        .style(move |style| {
            style
                .size(BUTTON_SIZE, BUTTON_SIZE)
                .items_center()
                .justify_center()
                .border_radius(styles::BUBBLE_RADIUS)
                .background(Color::TRANSPARENT)
        })
    }
}

fn tooltip_label(label: String) -> impl IntoView {
    text(label).style(|style| {
        style
            .padding_horiz(styles::TOOLTIP_PADDING_X)
            .padding_vert(styles::TOOLTIP_PADDING_Y)
            .border_radius(styles::TOOLTIP_RADIUS)
            .font_size(styles::FONT_META)
            .color(Color::WHITE)
            .background(styles::COLOR_TEXT)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn action_icon_button_uses_borderless_svg_container() {
        let source = include_str!("action_button.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("");

        assert!(source.contains("container(svg"));
        assert!(source.contains(".on_click_stop(move |_|"));
        assert!(source.contains(".background(Color::TRANSPARENT)"));
        assert!(!source.contains(".border(1.0)"));
    }
}
