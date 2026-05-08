use gpui::{Div, div, prelude::*, px, rgb};
use katana_chat_ui::{ChatUiColorSpec, ChatUiLayoutSpec};

const ICON_BUTTON_SIZE: f32 = 38.0;

pub(super) struct GpuiStyles;

impl GpuiStyles {
    pub(super) const COLORS: ChatUiColorSpec = ChatUiColorSpec::DEFAULT;
    pub(super) const LAYOUT: ChatUiLayoutSpec = ChatUiLayoutSpec::DEFAULT;
    pub(super) const CONTROL_ROW_GAP: f32 = 10.0;
    pub(super) const ICON_BUTTON_SIZE: f32 = ICON_BUTTON_SIZE;
    pub(super) const TOOLBAR_BUTTON_SIZE: f32 = 28.0;

    pub(super) fn centered(child: Div) -> Div {
        div().flex().w_full().justify_center().child(
            child
                .max_w(Self::px(Self::LAYOUT.chat_body_max_width))
                .w_full(),
        )
    }

    pub(super) fn icon_button(label: impl Into<String>) -> Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(Self::px(Self::ICON_BUTTON_SIZE))
            .h(Self::px(Self::ICON_BUTTON_SIZE))
            .rounded_full()
            .child(label.into())
    }

    pub(super) fn action_symbol(asset_id: &str) -> &'static str {
        match asset_id {
            "attach" => "📎",
            "history" => "↺",
            "new-chat" => "+",
            "send" => "↑",
            "settings" => "⚙",
            "stop" => "■",
            _ => "•",
        }
    }

    pub(super) fn provider_symbol(asset_id: &str) -> &'static str {
        match asset_id {
            "provider:claude-code" => "✺",
            "provider:codex-cli" => "◎",
            "provider:github-copilot" => "◉",
            "provider:ollama" => "⌁",
            "provider:opencode" => "▣",
            _ => "•",
        }
    }

    pub(super) fn color(rgb_value: [u8; 3]) -> gpui::Rgba {
        rgb(((rgb_value[0] as u32) << 16) | ((rgb_value[1] as u32) << 8) | rgb_value[2] as u32)
    }

    pub(super) fn px(value: f32) -> gpui::Pixels {
        px(value)
    }
}
