use super::styles;
use floem::{peniko::Color, prelude::*, unit::DurationUnitExt};

const THINKING_DOT_GAP: f64 = 6.0;
const THINKING_DOT_SIZE: f64 = 5.0;
const THINKING_DOT_ACTIVE_SIZE: f64 = 8.0;
const THINKING_DOT_RADIUS: f64 = 4.0;
const THINKING_DOT_IDLE_ALPHA: f32 = 0.35;
const THINKING_DOT_ACTIVE_ALPHA: f32 = 0.95;
const THINKING_DOT_DURATION_MS: u64 = 700;

pub(super) struct FloemThinkingIndicator;

impl FloemThinkingIndicator {
    pub(super) fn render(label_text: String, trailing: bool) -> impl IntoView {
        h_stack((text(label_text), dot(), dot(), dot())).style(move |style| {
            style
                .width_pct(styles::BUBBLE_WIDTH_PERCENT)
                .max_width(styles::BUBBLE_MAX_WIDTH)
                .min_width(0.0)
                .flex_shrink(1.0)
                .padding_horiz(styles::BUBBLE_PADDING_X)
                .padding_vert(styles::BUBBLE_PADDING_Y)
                .gap(THINKING_DOT_GAP)
                .items_center()
                .border_radius(styles::BUBBLE_RADIUS)
                .border(1.0)
                .border_color(styles::COLOR_BORDER)
                .background(background(trailing))
                .color(styles::COLOR_TEXT)
        })
    }
}

fn dot() -> impl IntoView {
    empty()
        .style(|style| {
            style
                .size(THINKING_DOT_SIZE, THINKING_DOT_SIZE)
                .border_radius(THINKING_DOT_RADIUS)
                .background(styles::COLOR_MUTED.multiply_alpha(THINKING_DOT_IDLE_ALPHA))
        })
        .animation(|animation| {
            animation
                .duration(THINKING_DOT_DURATION_MS.millis())
                .keyframe(0, |frame| frame.computed_style())
                .keyframe(100, |frame| {
                    frame.style(|style| {
                        style
                            .size(THINKING_DOT_ACTIVE_SIZE, THINKING_DOT_ACTIVE_SIZE)
                            .background(
                                styles::COLOR_MUTED.multiply_alpha(THINKING_DOT_ACTIVE_ALPHA),
                            )
                    })
                })
                .auto_reverse(true)
                .repeat(true)
        })
}

fn background(trailing: bool) -> Color {
    if trailing {
        return styles::COLOR_USER;
    }
    styles::COLOR_ASSISTANT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thinking_indicator_uses_user_background_for_trailing_message() {
        assert_eq!(background(true), styles::COLOR_USER);
    }

    #[test]
    fn thinking_indicator_uses_assistant_background_for_leading_message() {
        assert_eq!(background(false), styles::COLOR_ASSISTANT);
    }
}
