use gpui::{
    App, Application, Bounds, Context, TitlebarOptions, Window, WindowBounds, WindowOptions,
    prelude::*, px, size,
};
use katana_chat_ui::{
    ChatSession, ChatSessionError, ChatUiSurface, ContextUsageSnapshot, VendorUiState,
};
use katana_chat_ui_gpui::GpuiChatView;

const HARNESS_WINDOW_WIDTH: f32 = 1280.0;
const HARNESS_WINDOW_HEIGHT: f32 = 900.0;

struct ManualGpuiHost {
    view: GpuiChatView,
}

impl ManualGpuiHost {
    fn new() -> Result<Self, ChatSessionError> {
        Ok(Self {
            view: GpuiChatView::new(ChatUiSurface::from_render_model(
                &Self::session()?.render_model(),
            )),
        })
    }

    fn session() -> Result<ChatSession, ChatSessionError> {
        let mut session = ChatSession::new();
        session.set_title("katana-chat-ui");
        session.set_provider_configured("Claude Code");
        session.set_context_usage(ContextUsageSnapshot::new(0, 200_000));
        session.set_vendor_ui_state(manual_vendor_state());
        Ok(session)
    }
}

fn manual_vendor_state() -> VendorUiState {
    VendorUiState::for_vendor("claude-code")
        .with_available_vendors(available_vendor_ids())
        .with_models(claude_models(), "claude-sonnet-4-6")
        .with_thinking(claude_thinking_modes(), "default")
        .with_permission_modes(claude_permission_modes(), "default")
}

fn available_vendor_ids() -> Vec<String> {
    ["claude-code", "codex-cli", "github-copilot", "opencode"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn claude_models() -> Vec<String> {
    ["claude-sonnet-4-6", "claude-haiku-4-5"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn claude_thinking_modes() -> Vec<String> {
    ["default", "low", "medium", "high", "xhigh", "max"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn claude_permission_modes() -> Vec<String> {
    ["default", "auto", "plan"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

impl Render for ManualGpuiHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.view.render()
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let host = match ManualGpuiHost::new() {
            Ok(host) => host,
            Err(error) => {
                eprintln!("manual-host-gpui failed to build model: {error}");
                return;
            }
        };
        let bounds = Bounds::centered(
            None,
            size(px(HARNESS_WINDOW_WIDTH), px(HARNESS_WINDOW_HEIGHT)),
            cx,
        );
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("katana-chat-ui harness gpui".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = cx.open_window(options, |_, cx| cx.new(|_| host));
        if let Err(error) = result {
            eprintln!("manual-host-gpui failed to start: {error}");
            return;
        }
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_WINDOW_HEIGHT, HARNESS_WINDOW_WIDTH};

    #[test]
    fn harness_window_size_matches_floem_baseline() {
        assert_eq!(HARNESS_WINDOW_WIDTH, 1280.0);
        assert_eq!(HARNESS_WINDOW_HEIGHT, 900.0);
    }
}
