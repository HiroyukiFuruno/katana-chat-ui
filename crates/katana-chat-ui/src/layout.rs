#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatUiLayoutSpec {
    pub surface_padding: f32,
    pub root_gap: f32,
    pub panel_gap: f32,
    pub thread_padding: f32,
    pub message_gap: f32,
    pub chat_body_max_width: f32,
    pub agent_bubble_width_percent: f32,
    pub user_bubble_max_width: f32,
    pub bubble_padding_x: f32,
    pub bubble_padding_y: f32,
    pub bubble_radius: f32,
    pub composer_padding: f32,
    pub composer_input_height: f32,
    pub icon_size: f32,
    pub font_body: f32,
    pub font_meta: f32,
    pub toolbar_font_size: f32,
}

impl ChatUiLayoutSpec {
    pub const DEFAULT: Self = Self {
        surface_padding: 24.0,
        root_gap: 16.0,
        panel_gap: 14.0,
        thread_padding: 20.0,
        message_gap: 18.0,
        chat_body_max_width: 1600.0,
        agent_bubble_width_percent: 100.0,
        user_bubble_max_width: 720.0,
        bubble_padding_x: 16.0,
        bubble_padding_y: 2.0,
        bubble_radius: 16.0,
        composer_padding: 14.0,
        composer_input_height: 92.0,
        icon_size: 18.0,
        font_body: 15.0,
        font_meta: 12.0,
        toolbar_font_size: 18.0,
    };
}

const COLOR_CHANNELS: usize = 3;

pub type RgbColor = [u8; COLOR_CHANNELS];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatUiColorSpec {
    pub text: RgbColor,
    pub muted: RgbColor,
    pub panel: RgbColor,
    pub border: RgbColor,
    pub user: RgbColor,
    pub assistant: RgbColor,
}

impl ChatUiColorSpec {
    pub const DEFAULT: Self = Self {
        text: [32, 35, 39],
        muted: [103, 110, 118],
        panel: [246, 247, 249],
        border: [218, 223, 230],
        user: [232, 239, 249],
        assistant: [255, 255, 255],
    };
}

#[cfg(test)]
mod tests {
    use super::ChatUiLayoutSpec;

    #[test]
    fn default_layout_keeps_standard_chat_width_contract() {
        let layout = ChatUiLayoutSpec::DEFAULT;

        assert_eq!(layout.chat_body_max_width, 1600.0);
        assert_eq!(layout.agent_bubble_width_percent, 100.0);
        assert_eq!(layout.user_bubble_max_width, 720.0);
        assert_eq!(layout.bubble_padding_y, 2.0);
    }
}
