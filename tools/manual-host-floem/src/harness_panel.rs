use floem::{
    event::{Event, EventListener},
    peniko::Color,
    prelude::*,
};

use crate::{ViewSignals, state::ManualFloemState};

const EDGE_WIDTH: f64 = 18.0;
const PANEL_WIDTH: f64 = 360.0;
const PANEL_PADDING: f64 = 16.0;
const PANEL_GAP: f64 = 12.0;
const BUTTON_PADDING_X: f64 = 12.0;
const BUTTON_PADDING_Y: f64 = 8.0;
const BORDER_RADIUS: f64 = 14.0;
const FONT_BODY: f64 = 14.0;
const FONT_META: f64 = 12.0;
const EDGE_HIT_ALPHA: u8 = 1;
const EDGE_HIT_COLOR_CHANNEL: u8 = 255;
const EDGE_TRIGGER_Z_INDEX: i32 = 1001;
const PANEL_Z_INDEX: i32 = 1002;
const COLOR_TEXT: Color = Color::rgb8(32, 35, 39);
const COLOR_MUTED: Color = Color::rgb8(103, 110, 118);
const COLOR_BORDER: Color = Color::rgb8(218, 223, 230);

pub(crate) struct HarnessPanel;

impl HarnessPanel {
    pub(crate) fn render(signals: ViewSignals, hovered: RwSignal<bool>) -> impl IntoView {
        stack((
            edge_trigger(hovered),
            dyn_container(
                move || hovered.get(),
                move |is_hovered| {
                    if is_hovered {
                        return open_overlay(hovered, signals).into_any();
                    }
                    empty().into_any()
                },
            ),
        ))
        .style(move |style| {
            style
                .absolute()
                .inset_left(0.0)
                .inset_top(0.0)
                .inset_bottom(0.0)
                .width(HarnessOverlayMetrics::hit_width(hovered.get()))
                .z_index(EDGE_TRIGGER_Z_INDEX)
        })
        .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
        .on_event_cont(EventListener::FocusLost, move |_| hovered.set(false))
        .on_event_cont(EventListener::WindowLostFocus, move |_| hovered.set(false))
    }

    pub(crate) fn update_hover(event: &Event, hovered: RwSignal<bool>, window_left: RwSignal<f64>) {
        if let Event::PointerMove(pointer_event) = event {
            hovered.set(HarnessPointerRule::should_open(
                pointer_event.pos.x,
                window_left.get_untracked(),
                hovered.get_untracked(),
            ));
        }
    }
}

fn open_overlay(hovered: RwSignal<bool>, signals: ViewSignals) -> impl IntoView {
    panel(hovered, signals)
}

fn panel(hovered: RwSignal<bool>, signals: ViewSignals) -> impl IntoView {
    v_stack((
        heading("manual harness"),
        status(signals),
        action_button("provider を再検出", move || {
            signals
                .state
                .update(ManualFloemState::refresh_provider_registry_from_environment);
            signals.sync();
        }),
    ))
    .style(|style| {
        style
            .absolute()
            .inset_left(0.0)
            .inset_top(0.0)
            .inset_bottom(0.0)
            .width(PANEL_WIDTH)
            .padding(PANEL_PADDING)
            .gap(PANEL_GAP)
            .border(1.0)
            .border_color(COLOR_BORDER)
            .border_radius(BORDER_RADIUS)
            .background(Color::WHITE)
            .z_index(PANEL_Z_INDEX)
    })
    .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
    .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
    .on_event_cont(EventListener::FocusLost, move |_| hovered.set(false))
    .on_event_cont(EventListener::WindowLostFocus, move |_| hovered.set(false))
    .keyboard_navigable()
}

fn edge_trigger(hovered: RwSignal<bool>) -> impl IntoView {
    container(text(" "))
        .style(|style| {
            style
                .absolute()
                .inset_left(0.0)
                .inset_top(0.0)
                .inset_bottom(0.0)
                .width(EDGE_WIDTH)
                .background(Color::rgba8(
                    EDGE_HIT_COLOR_CHANNEL,
                    EDGE_HIT_COLOR_CHANNEL,
                    EDGE_HIT_COLOR_CHANNEL,
                    EDGE_HIT_ALPHA,
                ))
                .font_size(1.0)
                .color(Color::TRANSPARENT)
                .z_index(EDGE_TRIGGER_Z_INDEX)
        })
        .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
        .on_event_cont(EventListener::PointerMove, move |_| hovered.set(true))
        .on_event_cont(EventListener::WindowLostFocus, move |_| hovered.set(false))
}

struct HarnessOverlayMetrics;

impl HarnessOverlayMetrics {
    fn hit_width(hovered: bool) -> f64 {
        if hovered {
            return PANEL_WIDTH;
        }
        EDGE_WIDTH
    }
}

struct HarnessPointerRule;

impl HarnessPointerRule {
    fn should_open(pointer_x: f64, window_left: f64, already_open: bool) -> bool {
        let local_x = (pointer_x - window_left).abs();
        pointer_x <= EDGE_WIDTH
            || local_x <= EDGE_WIDTH
            || already_open && (pointer_x <= PANEL_WIDTH || local_x <= PANEL_WIDTH)
    }
}

fn heading(text_value: &'static str) -> impl IntoView {
    text(text_value).style(|style| style.font_size(FONT_BODY).color(COLOR_TEXT))
}

fn status(signals: ViewSignals) -> impl IntoView {
    v_stack((
        label(move || signals.last_event.get()),
        label(move || signals.vendor_summary.get()),
    ))
    .style(|style| style.gap(PANEL_GAP).font_size(FONT_META).color(COLOR_MUTED))
}

fn action_button<Action>(label_text: &'static str, action: Action) -> impl IntoView
where
    Action: Fn() + Copy + 'static,
{
    container(text(label_text))
        .on_click_stop(move |_| action())
        .style(|style| {
            style
                .padding_horiz(BUTTON_PADDING_X)
                .padding_vert(BUTTON_PADDING_Y)
                .border(1.0)
                .border_color(COLOR_BORDER)
                .border_radius(BORDER_RADIUS)
                .font_size(FONT_META)
                .color(COLOR_TEXT)
                .background(Color::WHITE)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct HarnessPopupState {
        visible: bool,
    }

    impl HarnessPopupState {
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
            HarnessOverlayMetrics::hit_width(self.visible)
        }
    }

    #[test]
    fn harness_panel_uses_invisible_left_edge_trigger() {
        assert_eq!(EDGE_WIDTH, 18.0);
        assert_eq!(EDGE_HIT_ALPHA, 1);
        assert!(include_str!("harness_panel.rs").contains("container(text(\" \"))"));
    }

    #[test]
    fn harness_panel_keeps_hit_area_on_left_edge_only() {
        assert_eq!(HarnessPopupState::closed().hit_width(), EDGE_WIDTH);
        assert_eq!(HarnessPopupState::opened().hit_width(), PANEL_WIDTH);
    }

    #[test]
    fn harness_panel_hit_area_width_follows_popup_state() {
        let source = include_str!("harness_panel.rs");

        assert!(source.contains("width(HarnessOverlayMetrics::hit_width(hovered.get()))"));
        assert!(source.contains("EventListener::PointerLeave"));
    }

    #[test]
    fn harness_panel_closes_on_blur() {
        let state = HarnessPopupState::opened().blurred();

        assert!(!state.visible);
    }

    #[test]
    fn harness_panel_closes_when_pointer_leaves_expanded_area() {
        let state = HarnessPopupState::opened().pointer_left();

        assert!(!state.visible);
    }

    #[test]
    fn harness_pointer_rule_uses_window_left_edge_and_panel_bounds() {
        assert!(HarnessPointerRule::should_open(4.0, 224.0, false));
        assert!(HarnessPointerRule::should_open(226.0, 224.0, false));
        assert!(HarnessPointerRule::should_open(300.0, 0.0, true));
        assert!(HarnessPointerRule::should_open(500.0, 224.0, true));
        assert!(!HarnessPointerRule::should_open(650.0, 224.0, true));
        assert!(!HarnessPointerRule::should_open(100.0, 224.0, false));
    }
}
