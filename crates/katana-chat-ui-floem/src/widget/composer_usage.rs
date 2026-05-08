use super::styles;
use floem::{peniko::Color, prelude::*};
use katana_chat_ui::ChatUiUsageSurface;

const USAGE_ICON_SIZE: f64 = 38.0;
const USAGE_ICON_RADIUS: f64 = 19.0;
const USAGE_TOOLTIP_WIDTH: f64 = 220.0;
const SVG_VIEWBOX_SIZE: f64 = 38.0;
const SVG_CENTER: f64 = 19.0;
const SVG_RADIUS: f64 = 15.0;
const SVG_STROKE_WIDTH: f64 = 3.0;
const SVG_CIRCUMFERENCE: f64 = 94.25;
const SVG_BACKGROUND_STROKE: &str = "#d8dee8";
const SVG_PROGRESS_STROKE: &str = "#24292f";
const MAX_PERCENTAGE: u8 = 100;

pub(super) struct FloemComposerUsageView;

impl FloemComposerUsageView {
    pub(super) fn render(usage: ChatUiUsageSurface) -> impl IntoView {
        let detail = ComposerUsagePresenter::detail(&usage);
        let percentage = ComposerUsagePresenter::percentage_value(&usage);
        let percentage_label = ComposerUsagePresenter::percentage(&usage);
        stack((pie_chart(percentage), percentage_overlay(percentage_label)))
            .tooltip(move || tooltip(detail.clone()))
            .style(|style| {
                style
                    .size(USAGE_ICON_SIZE, USAGE_ICON_SIZE)
                    .position(floem::style::Position::Relative)
                    .items_center()
                    .justify_center()
                    .border_radius(USAGE_ICON_RADIUS)
                    .background(Color::WHITE)
            })
    }
}

fn pie_chart(percentage: u8) -> impl IntoView {
    svg(UsagePieChartSvg::render(percentage)).style(|style| {
        style
            .size(USAGE_ICON_SIZE, USAGE_ICON_SIZE)
            .color(styles::COLOR_TEXT)
    })
}

fn percentage_overlay(percentage_label: String) -> impl IntoView {
    container(
        text(percentage_label)
            .style(|style| style.font_size(styles::FONT_META).color(styles::COLOR_TEXT)),
    )
    .style(|style| {
        style
            .absolute()
            .inset_left(0.0)
            .inset_top(0.0)
            .size(USAGE_ICON_SIZE, USAGE_ICON_SIZE)
            .items_center()
            .justify_center()
    })
}

struct ComposerUsagePresenter;

impl ComposerUsagePresenter {
    fn percentage(usage: &ChatUiUsageSurface) -> String {
        if usage.max_tokens == 0 {
            return "--%".to_string();
        }
        format!("{}%", Self::percentage_value(usage))
    }

    fn percentage_value(usage: &ChatUiUsageSurface) -> u8 {
        if usage.max_tokens == 0 {
            return 0;
        }
        usage.context_percentage.min(MAX_PERCENTAGE)
    }

    fn detail(usage: &ChatUiUsageSurface) -> String {
        if usage.max_tokens == 0 {
            return "コンテキスト画面:\n使用率 --%\n使用済み token: 不明\n\n提供元（provider）ごとに上限と管理方法が変わります"
                .to_string();
        }
        format!(
            "コンテキスト画面:\n使用率 {}%\n使用済み token: {} / {}\n\n提供元（provider）ごとに上限と管理方法が変わります\nstatus: {}\naccount: {}",
            usage.context_percentage,
            usage.used_tokens,
            usage.max_tokens,
            usage.context_status,
            usage.account_label
        )
    }
}

struct UsagePieChartSvg;

impl UsagePieChartSvg {
    fn render(percentage: u8) -> String {
        let filled = SVG_CIRCUMFERENCE * f64::from(percentage) / f64::from(MAX_PERCENTAGE);
        let remaining = SVG_CIRCUMFERENCE - filled;
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" data-kcu-icon="context-usage" viewBox="0 0 {viewbox} {viewbox}" fill="none">
<circle cx="{center}" cy="{center}" r="{radius}" stroke="{background}" stroke-width="{stroke_width}"/>
<circle cx="{center}" cy="{center}" r="{radius}" stroke="{progress}" stroke-width="{stroke_width}" stroke-linecap="round" stroke-dasharray="{filled:.2} {remaining:.2}" transform="rotate(-90 {center} {center})"/>
</svg>"##,
            viewbox = SVG_VIEWBOX_SIZE,
            center = SVG_CENTER,
            radius = SVG_RADIUS,
            background = SVG_BACKGROUND_STROKE,
            progress = SVG_PROGRESS_STROKE,
            stroke_width = SVG_STROKE_WIDTH
        )
    }
}

fn tooltip(detail: String) -> impl IntoView {
    label(move || detail.clone()).style(|style| {
        style
            .width(USAGE_TOOLTIP_WIDTH)
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
    use super::{ChatUiUsageSurface, ComposerUsagePresenter, UsagePieChartSvg};

    const USED_TOKENS: u64 = 750;
    const MAX_TOKENS: u64 = 1_000;
    const CONTEXT_PERCENTAGE: u8 = 75;
    const OVER_PERCENTAGE: u8 = 150;

    #[test]
    fn usage_icon_uses_pie_chart_svg() {
        let svg = UsagePieChartSvg::render(CONTEXT_PERCENTAGE);

        assert!(svg.contains(r#"data-kcu-icon="context-usage""#));
        assert!(svg.contains(r#"stroke-dasharray="70.69 23.56""#));
    }

    #[test]
    fn usage_percentage_is_clamped_for_chart() {
        let usage = ChatUiUsageSurface {
            used_tokens: USED_TOKENS,
            max_tokens: MAX_TOKENS,
            context_percentage: OVER_PERCENTAGE,
            context_status: "Warning".to_string(),
            account_label: "unavailable".to_string(),
        };

        assert_eq!(ComposerUsagePresenter::percentage(&usage), "100%");
        assert_eq!(ComposerUsagePresenter::percentage_value(&usage), 100);
    }

    #[test]
    fn usage_tooltip_keeps_provider_specific_limits_from_surface() {
        let usage = ChatUiUsageSurface {
            used_tokens: USED_TOKENS,
            max_tokens: MAX_TOKENS,
            context_percentage: CONTEXT_PERCENTAGE,
            context_status: "Warning".to_string(),
            account_label: "unavailable".to_string(),
        };

        assert_eq!(
            ComposerUsagePresenter::detail(&usage),
            "コンテキスト画面:\n使用率 75%\n使用済み token: 750 / 1000\n\n提供元（provider）ごとに上限と管理方法が変わります\nstatus: Warning\naccount: unavailable"
        );
    }

    #[test]
    fn usage_icon_handles_unavailable_context() {
        let usage = ChatUiUsageSurface {
            used_tokens: 0,
            max_tokens: 0,
            context_percentage: 0,
            context_status: "Unavailable".to_string(),
            account_label: "unavailable".to_string(),
        };

        assert_eq!(ComposerUsagePresenter::percentage(&usage), "--%");
        assert_eq!(
            ComposerUsagePresenter::detail(&usage),
            "コンテキスト画面:\n使用率 --%\n使用済み token: 不明\n\n提供元（provider）ごとに上限と管理方法が変わります"
        );
    }
}
