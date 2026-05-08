use super::styles;
use floem::{peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiVendorControlSurface, VendorOption};
use std::fmt;

const CONTROL_PADDING_X: f64 = 10.0;
const CONTROL_PADDING_Y: f64 = 4.0;
const CONTROL_RADIUS: f64 = 10.0;
const MAX_CONTROL_WIDTH: f64 = 220.0;
const DROPDOWN_HEIGHT: f64 = 38.0;
const SELECTOR_ICON_DELTA: f64 = 4.0;
const SELECTOR_GAP: f64 = 8.0;
const CHEVRON_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" data-kcu-icon="chevron-down" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>"#;

#[derive(Clone)]
pub(super) struct VendorChoice {
    pub(super) id: String,
    pub(super) label: String,
}

impl fmt::Display for VendorChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

#[derive(Clone)]
pub(super) struct ControlChoice {
    pub(super) value: String,
    pub(super) label: String,
}

impl fmt::Display for ControlChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

pub(super) struct VendorControlParts;

impl VendorControlParts {
    pub(super) fn vendor_options(options: &[VendorOption]) -> Vec<VendorChoice> {
        options
            .iter()
            .map(|it| VendorChoice {
                id: it.id.clone(),
                label: it.label.clone(),
            })
            .collect()
    }

    pub(super) fn control_choice(value: &str) -> ControlChoice {
        ControlChoice {
            value: value.to_string(),
            label: value.to_string(),
        }
    }

    pub(super) fn control_text(control: &ChatUiVendorControlSurface) -> String {
        format!("{}: {}", control.label, control.value)
    }

    pub(super) fn header_controls(
        controls: &[ChatUiVendorControlSurface],
    ) -> Vec<ChatUiVendorControlSurface> {
        controls
            .iter()
            .filter(|it| it.key != "endpoint")
            .cloned()
            .collect()
    }

    pub(super) fn static_chip(label: String, enabled: bool) -> impl IntoView {
        text(label).style(move |style| {
            style
                .max_width(MAX_CONTROL_WIDTH)
                .width(MAX_CONTROL_WIDTH)
                .padding_horiz(CONTROL_PADDING_X)
                .padding_vert(CONTROL_PADDING_Y)
                .border(1.0)
                .border_color(styles::COLOR_BORDER)
                .border_radius(CONTROL_RADIUS)
                .font_size(styles::FONT_META)
                .text_ellipsis()
                .flex_shrink(1.0)
                .color(text_color(enabled))
                .background(Color::WHITE)
        })
    }

    pub(super) fn selector_chip(label: String, enabled: bool) -> impl IntoView {
        h_stack((
            text(label).style(|style| style.text_ellipsis()),
            Self::selector_chevron(enabled),
        ))
        .style(move |style| selector_chip_style(style, enabled))
    }

    fn selector_chevron(enabled: bool) -> impl IntoView {
        svg(CHEVRON_ICON).style(move |style| {
            style
                .size(
                    styles::ICON_SIZE - SELECTOR_ICON_DELTA,
                    styles::ICON_SIZE - SELECTOR_ICON_DELTA,
                )
                .color(text_color(enabled))
                .flex_shrink(0.0)
        })
    }

    pub(super) fn tooltip_label(label: String) -> impl IntoView {
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
}

fn text_color(enabled: bool) -> Color {
    if enabled {
        return styles::COLOR_TEXT;
    }
    styles::COLOR_MUTED
}

fn selector_chip_style(style: floem::style::Style, enabled: bool) -> floem::style::Style {
    style
        .height(DROPDOWN_HEIGHT)
        .max_width(MAX_CONTROL_WIDTH)
        .width(MAX_CONTROL_WIDTH)
        .padding_horiz(CONTROL_PADDING_X)
        .padding_vert(CONTROL_PADDING_Y)
        .gap(SELECTOR_GAP)
        .items_center()
        .justify_between()
        .border(1.0)
        .border_color(styles::COLOR_BORDER)
        .border_radius(CONTROL_RADIUS)
        .font_size(styles::FONT_META)
        .flex_shrink(1.0)
        .color(text_color(enabled))
        .background(Color::WHITE)
}

#[cfg(test)]
mod tests;
