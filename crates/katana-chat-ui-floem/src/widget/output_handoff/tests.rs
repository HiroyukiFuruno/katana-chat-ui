use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OutputHandoffPopupState {
    visible: bool,
}

impl OutputHandoffPopupState {
    fn closed() -> Self {
        Self { visible: false }
    }

    fn opened() -> Self {
        Self { visible: true }
    }

    fn blurred(self) -> Self {
        Self { visible: false }
    }

    fn pointer_left(self) -> Self {
        Self { visible: false }
    }

    fn hit_width(self) -> f64 {
        OutputHandoffOverlayMetrics::hit_width(self.visible)
    }
}

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
    assert!(include_str!("../output_handoff.rs").contains("container(text(\" \"))"));
}

#[test]
fn hover_layer_width_follows_popup_state() {
    let source = include_str!("../output_handoff.rs");

    assert!(source.contains("width(OutputHandoffOverlayMetrics::hit_width(hovered.get()))"));
    assert!(source.contains("EventListener::PointerLeave"));
}

#[test]
fn output_hover_keeps_hit_area_on_right_edge_only() {
    assert_eq!(
        OutputHandoffPopupState::closed().hit_width(),
        EDGE_HOVER_WIDTH
    );
    assert_eq!(
        OutputHandoffPopupState::opened().hit_width(),
        POPUP_MAX_WIDTH
    );
}

#[test]
fn popup_state_closes_on_blur() {
    let state = OutputHandoffPopupState::opened().blurred();

    assert!(!state.visible);
}

#[test]
fn popup_state_closes_when_pointer_leaves_expanded_area() {
    let state = OutputHandoffPopupState::opened().pointer_left();

    assert!(!state.visible);
}

fn debug(enabled: bool) -> ChatUiDebugSurface {
    ChatUiDebugSurface {
        enabled,
        label: "Output".to_string(),
        output_handoff_text: r#"{ "kind": "DiffCandidate" }"#.to_string(),
    }
}
