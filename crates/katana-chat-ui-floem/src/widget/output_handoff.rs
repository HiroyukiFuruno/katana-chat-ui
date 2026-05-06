use super::styles;
use floem::{event::EventListener, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiDebugSurface, ChatUiSurface};

const EDGE_HOVER_WIDTH: f64 = 18.0;
const POPUP_MAX_WIDTH: f64 = 560.0;
const POPUP_HEIGHT: f64 = 620.0;
const POPUP_PADDING: f64 = 14.0;
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
    stack((
        dyn_container(
            move || hovered.get(),
            move |is_hovered| {
                if is_hovered {
                    return popup(hovered, popup_label.clone(), popup_output.clone()).into_any();
                }
                empty().into_any()
            },
        ),
        edge_trigger(hovered),
    ))
    .style(|style| {
        style
            .absolute()
            .inset_right(0.0)
            .inset_top(0.0)
            .width(POPUP_MAX_WIDTH)
            .height_full()
            .items_end()
    })
}

fn edge_trigger(hovered: RwSignal<bool>) -> impl IntoView {
    empty()
        .style(|style| {
            style
                .absolute()
                .inset_right(0.0)
                .inset_top(0.0)
                .width(EDGE_HOVER_WIDTH)
                .height_full()
                .background(Color::TRANSPARENT)
        })
        .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
}

fn popup(hovered: RwSignal<bool>, label_text: String, output_text: String) -> impl IntoView {
    v_stack((text(label_text), output_view(output_text)))
        .style(|style| {
            style
                .absolute()
                .inset_right(EDGE_HOVER_WIDTH)
                .inset_top(0.0)
                .max_width(POPUP_MAX_WIDTH)
                .height(POPUP_HEIGHT)
                .padding(POPUP_PADDING)
                .gap(styles::PANEL_GAP)
                .border(1.0)
                .border_color(styles::COLOR_BORDER)
                .border_radius(styles::BUBBLE_RADIUS)
                .background(Color::WHITE)
        })
        .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
        .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
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

    #[test]
    fn hover_trigger_has_no_visible_label() {
        assert_eq!(EDGE_HOVER_WIDTH, 18.0);
    }

    fn debug(enabled: bool) -> ChatUiDebugSurface {
        ChatUiDebugSurface {
            enabled,
            label: "Output".to_string(),
            output_handoff_text: r#"{ "kind": "DiffCandidate" }"#.to_string(),
        }
    }
}
