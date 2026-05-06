use super::styles;
use floem::{AnyView, peniko::Color, prelude::*, unit::DurationUnitExt};
use katana_chat_ui::ChatUiThinkingSurface;

const DETAIL_PADDING: f64 = 10.0;
const DOT_GAP: f64 = 5.0;
const DOT_SIZE: f64 = 4.0;
const DOT_ACTIVE_SIZE: f64 = 7.0;
const DOT_DURATION_MS: u64 = 700;
const DOT_IDLE_ALPHA: f32 = 0.35;
const DOT_ACTIVE_ALPHA: f32 = 0.95;
const THINKING_BACKGROUND: Color = Color::rgb8(255, 255, 255);

pub(super) struct FloemThinkingView;

impl FloemThinkingView {
    pub(super) fn render(thinking: ChatUiThinkingSurface) -> AnyView {
        let state = ThinkingViewState::from_surface(&thinking);
        let expanded = RwSignal::new(thinking.expanded);
        let details = thinking.entries.clone();
        container(v_stack((
            header(state.label, state.completed),
            detail_area(expanded, details),
        )))
        .on_click_stop(move |_| expanded.update(|it| *it = !*it))
        .style(|style| {
            style
                .width_full()
                .min_width(0.0)
                .padding(DETAIL_PADDING)
                .border(1.0)
                .border_color(styles::COLOR_BORDER)
                .border_radius(styles::BUBBLE_RADIUS)
                .background(THINKING_BACKGROUND)
        })
        .into_any()
    }
}

fn header(label_text: String, completed: bool) -> impl IntoView {
    h_stack((
        label(move || label_text.clone())
            .style(|style| style.font_size(styles::FONT_BODY).color(styles::COLOR_TEXT)),
        running_indicator(completed),
    ))
    .style(|style| style.items_center().gap(DOT_GAP))
}

fn running_indicator(completed: bool) -> AnyView {
    if completed {
        return empty().into_any();
    }
    h_stack((dot(), dot(), dot()))
        .style(|style| style.gap(DOT_GAP).items_center())
        .into_any()
}

fn detail_area(expanded: RwSignal<bool>, details: Vec<String>) -> impl IntoView {
    dyn_container(
        move || expanded.get(),
        move |is_expanded| {
            if is_expanded {
                return detail_text(details.clone()).into_any();
            }
            empty().into_any()
        },
    )
}

fn detail_text(details: Vec<String>) -> impl IntoView {
    label(move || details.join("\n")).style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .padding_top(DETAIL_PADDING)
            .font_size(styles::FONT_META)
            .color(styles::COLOR_MUTED)
    })
}

fn dot() -> impl IntoView {
    empty()
        .style(|style| {
            style
                .size(DOT_SIZE, DOT_SIZE)
                .border_radius(DOT_ACTIVE_SIZE)
                .background(styles::COLOR_MUTED.multiply_alpha(DOT_IDLE_ALPHA))
        })
        .animation(|animation| {
            animation
                .duration(DOT_DURATION_MS.millis())
                .keyframe(0, |frame| frame.computed_style())
                .keyframe(100, |frame| {
                    frame.style(|style| {
                        style
                            .size(DOT_ACTIVE_SIZE, DOT_ACTIVE_SIZE)
                            .background(styles::COLOR_MUTED.multiply_alpha(DOT_ACTIVE_ALPHA))
                    })
                })
                .auto_reverse(true)
                .repeat(true)
        })
}

#[derive(Debug, PartialEq, Eq)]
struct ThinkingViewState {
    label: String,
    details: Vec<String>,
    expanded: bool,
    completed: bool,
    indicator_visible: bool,
}

impl ThinkingViewState {
    fn from_surface(thinking: &ChatUiThinkingSurface) -> Self {
        Self {
            label: thinking.label.clone(),
            details: thinking.entries.clone(),
            expanded: thinking.expanded,
            completed: thinking.completed,
            indicator_visible: !thinking.completed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_thinking_state_keeps_entries_separate_from_answer_body() {
        let state = ThinkingViewState::from_surface(&surface(false));

        assert_eq!(state.label, "Thinking");
        assert_eq!(state.details, vec!["read context", "plan response"]);
        assert!(state.expanded);
        assert!(state.indicator_visible);
    }

    #[test]
    fn completed_thinking_state_hides_running_indicator() {
        let state = ThinkingViewState::from_surface(&surface(true));

        assert!(state.completed);
        assert!(!state.indicator_visible);
    }

    fn surface(completed: bool) -> ChatUiThinkingSurface {
        ChatUiThinkingSurface {
            label: "Thinking".to_string(),
            entries: vec!["read context".to_string(), "plan response".to_string()],
            expanded: !completed,
            completed,
        }
    }
}
