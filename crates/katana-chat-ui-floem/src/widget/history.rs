use super::styles;
use floem::{AnyView, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiHistoryPanelSurface, ChatUiHistorySessionSurface, ChatUiSurface};

const HISTORY_PANEL_MAX_WIDTH: f64 = 720.0;
const HISTORY_PANEL_PADDING: f64 = 14.0;
const HISTORY_ROW_PADDING_X: f64 = 12.0;
const HISTORY_ROW_PADDING_Y: f64 = 10.0;
const HISTORY_ROW_GAP: f64 = 4.0;
const HISTORY_PANEL_GAP: f64 = 10.0;
const HISTORY_ROW_BACKGROUND: Color = Color::rgb8(248, 249, 251);

pub(super) struct FloemHistoryPanelView;

impl FloemHistoryPanelView {
    pub(super) fn render<OnHistorySelect>(
        surface: RwSignal<ChatUiSurface>,
        on_history_select: OnHistorySelect,
    ) -> impl IntoView
    where
        OnHistorySelect: Fn(String) + Copy + 'static,
    {
        dyn_container(
            move || surface.get().history_panel,
            move |panel| panel_view(panel, on_history_select),
        )
    }
}

fn panel_view<OnHistorySelect>(
    panel: ChatUiHistoryPanelSurface,
    on_history_select: OnHistorySelect,
) -> AnyView
where
    OnHistorySelect: Fn(String) + Copy + 'static,
{
    if !panel.visible {
        return empty().into_any();
    }
    h_stack((history_panel_content(panel, on_history_select),))
        .style(|style| style.width_full().items_center().justify_center())
        .into_any()
}

fn history_panel_content<OnHistorySelect>(
    panel: ChatUiHistoryPanelSurface,
    on_history_select: OnHistorySelect,
) -> impl IntoView
where
    OnHistorySelect: Fn(String) + Copy + 'static,
{
    v_stack((
        panel_label(panel.label),
        history_sessions(panel.sessions, panel.empty_label, on_history_select),
    ))
    .style(|style| {
        style
            .width_pct(70.0)
            .max_width(HISTORY_PANEL_MAX_WIDTH)
            .min_width(0.0)
            .padding(HISTORY_PANEL_PADDING)
            .gap(HISTORY_PANEL_GAP)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::BUBBLE_RADIUS)
            .background(Color::WHITE)
    })
}

fn panel_label(label_text: String) -> impl IntoView {
    label(move || label_text.clone())
        .style(|style| style.font_size(styles::FONT_BODY).color(styles::COLOR_TEXT))
}

fn history_sessions<OnHistorySelect>(
    sessions: Vec<ChatUiHistorySessionSurface>,
    empty_label: String,
    on_history_select: OnHistorySelect,
) -> AnyView
where
    OnHistorySelect: Fn(String) + Copy + 'static,
{
    if sessions.is_empty() {
        return label(move || empty_label.clone())
            .style(|style| {
                style
                    .font_size(styles::FONT_META)
                    .color(styles::COLOR_MUTED)
            })
            .into_any();
    }
    v_stack_from_iter(
        sessions
            .into_iter()
            .map(move |session| history_session_row(session, on_history_select)),
    )
    .style(|style| style.width_full().min_width(0.0).gap(HISTORY_ROW_GAP))
    .into_any()
}

fn history_session_row<OnHistorySelect>(
    session: ChatUiHistorySessionSurface,
    on_history_select: OnHistorySelect,
) -> impl IntoView
where
    OnHistorySelect: Fn(String) + Copy + 'static,
{
    let session_id = session.session_id.clone();
    v_stack((
        row_title(session.title),
        row_meta(session.provider_label, session.updated_at_label),
        row_preview(session.preview),
    ))
    .on_click_stop(move |_| on_history_select(session_id.clone()))
    .style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .padding_horiz(HISTORY_ROW_PADDING_X)
            .padding_vert(HISTORY_ROW_PADDING_Y)
            .gap(HISTORY_ROW_GAP)
            .border_radius(styles::BUBBLE_RADIUS)
            .background(HISTORY_ROW_BACKGROUND)
    })
}

fn row_title(title: String) -> impl IntoView {
    label(move || title.clone())
        .style(|style| style.font_size(styles::FONT_BODY).color(styles::COLOR_TEXT))
}

fn row_meta(provider: String, updated_at: String) -> impl IntoView {
    label(move || format!("{provider} / {updated_at}")).style(|style| {
        style
            .font_size(styles::FONT_META)
            .color(styles::COLOR_MUTED)
    })
}

fn row_preview(preview: String) -> impl IntoView {
    label(move || preview.clone()).style(|style| {
        style
            .font_size(styles::FONT_META)
            .color(styles::COLOR_TEXT)
            .text_ellipsis()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn history_panel_is_driven_by_surface_state() {
        let source = include_str!("history.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("");

        assert!(source.contains("surface.get().history_panel"));
        assert!(source.contains("if !panel.visible"));
        assert!(source.contains(".on_click_stop(move |_| on_history_select"));
    }
}
