use katana_chat_ui::{
    Attachment, ChatInputDraft, ChatSession, ChatUiSurface, ContextUsageSnapshot, FileResource,
    VendorUiState,
};

pub(super) const STANDARD_WIDTH: u32 = 640;
pub(super) const STANDARD_HEIGHT: u32 = 480;

pub(super) fn standard_surface() -> Result<ChatUiSurface, String> {
    let mut session = configured_session();
    session.draft_mut().set_text("short");
    session.submit_draft().map_err(|error| error.to_string())?;
    session
        .start_assistant_stream(markdown())
        .map_err(|error| error.to_string())?;
    session
        .finish_assistant_message()
        .map_err(|error| error.to_string())?;
    session.draft_mut().set_text("submit this");
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

pub(super) fn attachment_surface() -> ChatUiSurface {
    let mut session = configured_session();
    let request = ChatInputDraft::request_path_drop("/tmp/kcu-fixture.md", "text/markdown");
    session
        .draft_mut()
        .add_attachment(Attachment::file(FileResource::new(
            request.uri,
            "text/markdown",
            "# Fixture",
            9,
        )));
    ChatUiSurface::from_render_model(&session.render_model())
}

fn configured_session() -> ChatSession {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.set_context_usage(ContextUsageSnapshot::new(750, 1000));
    session.set_vendor_ui_state(vendor_state());
    session
}

fn vendor_state() -> VendorUiState {
    VendorUiState::for_vendor("claude-code")
        .with_available_vendors(vec!["claude-code".to_string(), "codex-cli".to_string()])
        .with_models(vec!["claude-sonnet".to_string()], "claude-sonnet")
        .with_thinking(vec!["medium".to_string()], "medium")
        .with_permission_modes(vec!["default".to_string()], "default")
}

fn markdown() -> String {
    "<script>alert(1)</script>\n\n```rust\nlet x = 1;\n```\n\n| A | B |\n|---|---|\n| c | d |"
        .to_string()
}
