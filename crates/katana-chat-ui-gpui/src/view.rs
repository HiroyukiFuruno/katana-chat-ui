mod composer;
mod header;
mod styles;
mod thread;

use gpui::{Div, div, prelude::*};
use katana_chat_ui::ChatUiSurface;

use self::{
    composer::GpuiComposerView, header::GpuiHeaderView, styles::GpuiStyles, thread::GpuiThreadView,
};

pub struct GpuiChatView {
    surface: ChatUiSurface,
}

impl GpuiChatView {
    pub fn new(surface: ChatUiSurface) -> Self {
        Self { surface }
    }

    pub fn render(&self) -> Div {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(GpuiStyles::color(GpuiStyles::COLORS.panel))
            .text_color(GpuiStyles::color(GpuiStyles::COLORS.text))
            .p(GpuiStyles::px(GpuiStyles::LAYOUT.surface_padding))
            .gap(GpuiStyles::px(GpuiStyles::LAYOUT.root_gap))
            .child(GpuiHeaderView::render(&self.surface))
            .child(GpuiThreadView::render(&self.surface))
            .child(GpuiStyles::centered(GpuiComposerView::render(
                &self.surface,
            )))
    }
}

#[cfg(test)]
mod tests {
    use super::{GpuiChatView, styles::GpuiStyles};
    use katana_chat_ui::{ChatSession, ChatUiSurface, ContextUsageSnapshot, VendorUiState};

    #[test]
    fn render_accepts_standard_chat_surface() {
        let surface = sample_surface();
        let view = GpuiChatView::new(surface);
        let _rendered = view.render();
    }

    #[test]
    fn sample_surface_starts_without_harness_messages() {
        let surface = sample_surface();

        assert!(surface.message_list.messages.is_empty());
        assert_eq!(surface.vendor_bar.active_vendor_id, "claude-code");
    }

    #[test]
    fn provider_header_icon_uses_active_provider_asset() {
        assert_eq!(GpuiStyles::provider_symbol("provider:claude-code"), "✺");
    }

    fn sample_surface() -> ChatUiSurface {
        let mut session = ChatSession::new();
        session.set_title("katana-chat-ui");
        session.set_provider_configured("Claude Code");
        session.set_context_usage(ContextUsageSnapshot::new(0, 200_000));
        session.set_vendor_ui_state(
            VendorUiState::for_vendor("claude-code")
                .with_available_vendors(vec![
                    "claude-code".to_string(),
                    "codex-cli".to_string(),
                    "github-copilot".to_string(),
                    "opencode".to_string(),
                ])
                .with_models(vec!["claude-sonnet-4-6".to_string()], "claude-sonnet-4-6")
                .with_thinking(vec!["default".to_string()], "default")
                .with_permission_modes(vec!["default".to_string()], "default"),
        );
        ChatUiSurface::from_render_model(&session.render_model())
    }
}
