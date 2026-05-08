use super::styles;
use floem::{event::EventListener, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiDebugSurface, ChatUiSurface};

pub(super) const EDGE_HOVER_WIDTH: f64 = 18.0;
const POPUP_MAX_WIDTH: f64 = 560.0;
const POPUP_HEIGHT: f64 = 620.0;
const POPUP_PADDING: f64 = 14.0;
const OUTPUT_PADDING: f64 = 12.0;
const POPUP_Z_INDEX: i32 = 1002;
const EDGE_HIT_COLOR_CHANNEL: u8 = 255;
const EDGE_HIT_ALPHA: u8 = 1;

pub(super) struct FloemOutputHandoffHover;

impl FloemOutputHandoffHover {
    pub(super) fn render(
        surface: RwSignal<ChatUiSurface>,
        hovered: RwSignal<bool>,
    ) -> impl IntoView {
        stack((
            edge_trigger(hovered),
            dyn_container(
                move || (surface.get().debug, hovered.get()),
                move |(debug, is_hovered)| {
                    let state = OutputHandoffHoverState::from_debug(&debug);
                    if state.visible && is_hovered {
                        return popup(hovered, state.label, state.output_text).into_any();
                    }
                    empty().into_any()
                },
            ),
        ))
        .style(move |style| {
            style
                .absolute()
                .inset_right(0.0)
                .inset_top(0.0)
                .inset_bottom(0.0)
                .width(OutputHandoffOverlayMetrics::hit_width(hovered.get()))
                .z_index(POPUP_Z_INDEX)
        })
        .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
        .on_event_cont(EventListener::FocusLost, move |_| hovered.set(false))
        .on_event_cont(EventListener::WindowLostFocus, move |_| hovered.set(false))
    }
}

fn edge_trigger(hovered: RwSignal<bool>) -> impl IntoView {
    container(text(" "))
        .style(|style| {
            style
                .absolute()
                .inset_right(0.0)
                .inset_top(0.0)
                .width(EDGE_HOVER_WIDTH)
                .height_full()
                .background(Color::rgba8(
                    EDGE_HIT_COLOR_CHANNEL,
                    EDGE_HIT_COLOR_CHANNEL,
                    EDGE_HIT_COLOR_CHANNEL,
                    EDGE_HIT_ALPHA,
                ))
                .font_size(1.0)
                .color(Color::TRANSPARENT)
                .z_index(POPUP_Z_INDEX - 1)
        })
        .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
        .on_event_cont(EventListener::PointerMove, move |_| hovered.set(true))
        .on_event_cont(EventListener::WindowLostFocus, move |_| hovered.set(false))
}

fn popup(hovered: RwSignal<bool>, label_text: String, output_text: String) -> impl IntoView {
    v_stack((text(label_text), output_view(output_text)))
        .style(|style| {
            style
                .absolute()
                .inset_right(0.0)
                .inset_top(0.0)
                .width(POPUP_MAX_WIDTH)
                .height(POPUP_HEIGHT)
                .padding(POPUP_PADDING)
                .gap(styles::PANEL_GAP)
                .border(1.0)
                .border_color(styles::COLOR_BORDER)
                .border_radius(styles::BUBBLE_RADIUS)
                .background(Color::WHITE)
                .z_index(POPUP_Z_INDEX)
        })
        .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
        .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
        .on_event_cont(EventListener::FocusLost, move |_| hovered.set(false))
        .on_event_cont(EventListener::WindowLostFocus, move |_| hovered.set(false))
        .keyboard_navigable()
}

fn output_view(output_text: String) -> impl IntoView {
    scroll(label(move || output_text.clone())).style(|style| {
        style
            .width_full()
            .height_full()
            .padding(OUTPUT_PADDING)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::BUBBLE_RADIUS)
    })
}

#[derive(Debug, PartialEq, Eq)]
struct OutputHandoffHoverState {
    visible: bool,
    label: String,
    output_text: String,
}

impl OutputHandoffHoverState {
    fn from_debug(debug: &ChatUiDebugSurface) -> Self {
        Self {
            visible: debug.enabled,
            label: debug.label.clone(),
            output_text: debug.output_handoff_text.clone(),
        }
    }
}

struct OutputHandoffOverlayMetrics;

impl OutputHandoffOverlayMetrics {
    fn hit_width(hovered: bool) -> f64 {
        if hovered {
            return POPUP_MAX_WIDTH;
        }
        EDGE_HOVER_WIDTH
    }
}

#[cfg(test)]
mod tests;
