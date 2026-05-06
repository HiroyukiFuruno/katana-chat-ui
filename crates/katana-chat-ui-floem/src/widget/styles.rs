use floem::peniko::Color;

const SURFACE_PADDING: f64 = 24.0;
const ROOT_GAP: f64 = 16.0;
const TOOLBAR_FONT_SIZE: f64 = 18.0;
pub(super) const PANEL_GAP: f64 = 14.0;
pub(super) const THREAD_PADDING: f64 = 20.0;
pub(super) const MESSAGE_GAP: f64 = 18.0;
pub(super) const BUBBLE_WIDTH_PERCENT: f64 = 78.0;
pub(super) const BUBBLE_MAX_WIDTH: f64 = 980.0;
pub(super) const BUBBLE_PADDING_X: f64 = 16.0;
pub(super) const BUBBLE_PADDING_Y: f64 = 12.0;
pub(super) const BUBBLE_RADIUS: f64 = 16.0;
pub(super) const ICON_SIZE: f64 = 18.0;
pub(super) const FONT_BODY: f64 = 15.0;
pub(super) const FONT_META: f64 = 12.0;
pub(super) const TOOLTIP_PADDING_X: f64 = 8.0;
pub(super) const TOOLTIP_PADDING_Y: f64 = 4.0;
pub(super) const TOOLTIP_RADIUS: f64 = 6.0;
pub(super) const COLOR_TEXT: Color = Color::rgb8(32, 35, 39);
pub(super) const COLOR_MUTED: Color = Color::rgb8(103, 110, 118);
pub(super) const COLOR_PANEL: Color = Color::rgb8(246, 247, 249);
pub(super) const COLOR_BORDER: Color = Color::rgb8(218, 223, 230);
pub(super) const COLOR_USER: Color = Color::rgb8(232, 239, 249);
pub(super) const COLOR_ASSISTANT: Color = Color::rgb8(255, 255, 255);

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
}
