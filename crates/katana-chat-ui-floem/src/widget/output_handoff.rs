use super::styles;
use floem::{event::EventListener, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiDebugSurface, ChatUiSurface};

const TRIGGER_OFFSET: f64 = 8.0;
const TRIGGER_PADDING_X: f64 = 10.0;
const TRIGGER_PADDING_Y: f64 = 6.0;
const POPUP_MAX_WIDTH: f64 = 560.0;
const POPUP_HEIGHT: f64 = 620.0;
const POPUP_PADDING: f64 = 14.0;
const POPUP_GAP: f64 = 10.0;
const OUTPUT_PADDING: f64 = 12.0;

pub(super) struct FloemOutputHandoffHover;

impl FloemOutputHandoffHover {
    pub(super) fn render(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
        dyn_container(
            move || surface.get().debug,
            move |debug| {
                if OutputHandoffHoverState::from_debug(&debug).visible {
                    return trigger(debug).into_any();
                }
                empty().into_any()
            },
        )
    }
}

fn trigger(debug: ChatUiDebugSurface) -> impl IntoView {
    let hovered = RwSignal::new(false);
    let popup_label = debug.label.clone();
    let popup_output = debug.output_handoff_text.clone();
    let chip_label = debug.label;
    v_stack((
        dyn_container(
            move || hovered.get(),
            move |is_hovered| {
                if is_hovered {
                    return popup(popup_label.clone(), popup_output.clone()).into_any();
                }
                empty().into_any()
            },
        ),
        trigger_chip(chip_label),
    ))
    .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
    .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
    .style(|style| {
        style
            .absolute()
            .inset_right(TRIGGER_OFFSET)
            .inset_bottom(TRIGGER_OFFSET)
            .items_end()
            .gap(POPUP_GAP)
    })
}

fn trigger_chip(label: String) -> impl IntoView {
    container(text(label)).style(|style| {
        style
            .padding_horiz(TRIGGER_PADDING_X)
            .padding_vert(TRIGGER_PADDING_Y)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::BUBBLE_RADIUS)
            .font_size(styles::FONT_META)
            .background(Color::WHITE)
    })
}

fn popup(label_text: String, output_text: String) -> impl IntoView {
    v_stack((text(label_text), output_view(output_text))).style(|style| {
        style
            .width_full()
            .max_width(POPUP_MAX_WIDTH)
            .height(POPUP_HEIGHT)
            .padding(POPUP_PADDING)
            .gap(POPUP_GAP)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::BUBBLE_RADIUS)
            .background(Color::WHITE)
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_state_is_hidden_when_debug_is_disabled() {
        let state = OutputHandoffHoverState::from_debug(&debug(false));

        assert!(!state.visible);
        assert_eq!(state.label, "Output");
    }

    #[test]
    fn hover_state_keeps_output_json_when_debug_is_enabled() {
        let state = OutputHandoffHoverState::from_debug(&debug(true));

        assert!(state.visible);
        assert!(state.output_text.contains("\"kind\""));
    }

    fn debug(enabled: bool) -> ChatUiDebugSurface {
        ChatUiDebugSurface {
            enabled,
            label: "Output".to_string(),
            output_handoff_text: r#"{ "kind": "DiffCandidate" }"#.to_string(),
        }
    }
}
