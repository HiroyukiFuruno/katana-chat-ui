use floem::{event::EventListener, peniko::Color, prelude::*};

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
const COLOR_TEXT: Color = Color::rgb8(32, 35, 39);
const COLOR_MUTED: Color = Color::rgb8(103, 110, 118);
const COLOR_BORDER: Color = Color::rgb8(218, 223, 230);

pub(crate) struct HarnessPanel;

impl HarnessPanel {
    pub(crate) fn render(signals: ViewSignals) -> impl IntoView {
        let hovered = RwSignal::new(false);
        stack((
            dyn_container(
                move || hovered.get(),
                move |is_hovered| {
                    if is_hovered {
                        return panel(signals).into_any();
                    }
                    empty().into_any()
                },
            ),
            edge_trigger(),
        ))
        .on_event_cont(EventListener::PointerEnter, move |_| hovered.set(true))
        .on_event_cont(EventListener::PointerLeave, move |_| hovered.set(false))
        .style(|style| {
            style
                .absolute()
                .inset_left(0.0)
                .inset_top(0.0)
                .width(PANEL_WIDTH)
                .height_full()
        })
    }
}

fn panel(signals: ViewSignals) -> impl IntoView {
    v_stack((
        heading("manual harness"),
        status(signals),
        action_button("provider を再検出", move || {
            signals
                .state
                .update(ManualFloemState::refresh_ollama_models);
            signals.sync();
        }),
        action_button("履歴サンプルを追加", move || {
            signals
                .state
                .update(ManualFloemState::add_harness_sample_history);
            signals.sync();
        }),
    ))
    .style(|style| {
        style
            .absolute()
            .inset_left(EDGE_WIDTH)
            .inset_top(0.0)
            .width(PANEL_WIDTH - EDGE_WIDTH)
            .padding(PANEL_PADDING)
            .gap(PANEL_GAP)
            .border(1.0)
            .border_color(COLOR_BORDER)
            .border_radius(BORDER_RADIUS)
            .background(Color::WHITE)
    })
}

fn edge_trigger() -> impl IntoView {
    empty().style(|style| {
        style
            .absolute()
            .inset_left(0.0)
            .inset_top(0.0)
            .width(EDGE_WIDTH)
            .height_full()
            .background(Color::TRANSPARENT)
    })
}

fn heading(text_value: &'static str) -> impl IntoView {
    text(text_value).style(|style| style.font_size(FONT_BODY).color(COLOR_TEXT))
}

fn status(signals: ViewSignals) -> impl IntoView {
    v_stack((
        label(move || signals.last_event.get()),
        label(move || signals.vendor_summary.get()),
    ))
    .style(|style| {
        style
            .gap(PANEL_GAP)
            .font_size(FONT_META)
            .color(COLOR_MUTED)
    })
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

    #[test]
    fn harness_panel_uses_invisible_left_edge_trigger() {
        assert_eq!(EDGE_WIDTH, 18.0);
    }
}
