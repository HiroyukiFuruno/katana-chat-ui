use floem::peniko::Color;
use katana_chat_ui::{ChatUiColorSpec, ChatUiLayoutSpec};

const LAYOUT: ChatUiLayoutSpec = ChatUiLayoutSpec::DEFAULT;
const COLORS: ChatUiColorSpec = ChatUiColorSpec::DEFAULT;

const SURFACE_PADDING: f64 = LAYOUT.surface_padding as f64;
const ROOT_GAP: f64 = LAYOUT.root_gap as f64;
const TOOLBAR_FONT_SIZE: f64 = LAYOUT.toolbar_font_size as f64;
pub(super) const PANEL_GAP: f64 = LAYOUT.panel_gap as f64;
pub(super) const THREAD_PADDING: f64 = LAYOUT.thread_padding as f64;
pub(super) const MESSAGE_GAP: f64 = LAYOUT.message_gap as f64;
pub(super) const CHAT_BODY_MAX_WIDTH: f64 = LAYOUT.chat_body_max_width as f64;
pub(super) const AGENT_BUBBLE_WIDTH_PERCENT: f64 = LAYOUT.agent_bubble_width_percent as f64;
pub(super) const AGENT_BUBBLE_MAX_WIDTH: f64 = CHAT_BODY_MAX_WIDTH;
pub(super) const USER_BUBBLE_MAX_WIDTH: f64 = LAYOUT.user_bubble_max_width as f64;
pub(super) const BUBBLE_PADDING_X: f64 = LAYOUT.bubble_padding_x as f64;
pub(super) const BUBBLE_PADDING_Y: f64 = LAYOUT.bubble_padding_y as f64;
pub(super) const BUBBLE_RADIUS: f64 = LAYOUT.bubble_radius as f64;
pub(super) const ICON_SIZE: f64 = LAYOUT.icon_size as f64;
pub(super) const FONT_BODY: f64 = LAYOUT.font_body as f64;
pub(super) const FONT_META: f64 = LAYOUT.font_meta as f64;
pub(super) const TOOLTIP_PADDING_X: f64 = 8.0;
pub(super) const TOOLTIP_PADDING_Y: f64 = 4.0;
pub(super) const TOOLTIP_RADIUS: f64 = 6.0;
pub(super) const COLOR_TEXT: Color = Color::rgb8(COLORS.text[0], COLORS.text[1], COLORS.text[2]);
pub(super) const COLOR_MUTED: Color =
    Color::rgb8(COLORS.muted[0], COLORS.muted[1], COLORS.muted[2]);
pub(super) const COLOR_PANEL: Color =
    Color::rgb8(COLORS.panel[0], COLORS.panel[1], COLORS.panel[2]);
pub(super) const COLOR_BORDER: Color =
    Color::rgb8(COLORS.border[0], COLORS.border[1], COLORS.border[2]);
pub(super) const COLOR_USER: Color = Color::rgb8(COLORS.user[0], COLORS.user[1], COLORS.user[2]);
pub(super) const COLOR_ASSISTANT: Color = Color::rgb8(
    COLORS.assistant[0],
    COLORS.assistant[1],
    COLORS.assistant[2],
);

pub(super) struct FloemWidgetStyle;

impl FloemWidgetStyle {
    pub(super) fn root(style: floem::style::Style) -> floem::style::Style {
        style
            .size_full()
            .padding(SURFACE_PADDING)
            .gap(ROOT_GAP)
            .flex_col()
            .background(COLOR_PANEL)
    }

    pub(super) fn toolbar_title(style: floem::style::Style) -> floem::style::Style {
        style.font_size(TOOLBAR_FONT_SIZE).color(COLOR_TEXT)
    }

    pub(super) fn provider_selector_label(style: floem::style::Style) -> floem::style::Style {
        style
            .font_size(FONT_BODY)
            .color(COLOR_TEXT)
            .flex_shrink(0.0)
    }
}
