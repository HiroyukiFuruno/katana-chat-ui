use super::ChatPanelView;
use crate::{FloemChatActions, FloemChatPanel, FloemChatView, FloemPanelSlot};
use floem::prelude::*;
use katana_chat_ui::{
    Attachment, ChatLocale, ChatOutputKind, ChatSession, ChatTextKey, ChatUiSurface,
    ContextUsageSnapshot, FileCandidateOutput, FileResource, HostActionKind, MessageRole, SvgIcon,
    TextCatalog, UsageStatus, VendorUiState,
};

#[test]
fn reference_panel_reads_only_chat_render_model() -> Result<(), katana_chat_ui::ChatSessionError> {
    let mut session = sample_session()?;
    add_sample_output(&mut session)?;

    let view = ChatPanelView::from_render_model(&session.render_model());

    assert!(view.composer.multiline_enabled);
    assert!(view.composer.ime_enabled);
    assert_eq!(view.message_list.messages[0].role, MessageRole::User);
    assert_eq!(view.message_list.messages[0].attachment_count, 1);
    assert_eq!(
        view.output_list.outputs[0].actions[0],
        HostActionKind::OpenPreview
    );
    assert_eq!(view.usage_meter.context_status, UsageStatus::Critical);
    Ok(())
}

#[test]
fn standard_widget_builds_from_chat_surface() -> Result<(), katana_chat_ui::ChatSessionError> {
    let session = sample_session()?;
    let surface = RwSignal::new(ChatUiSurface::from_render_model(&session.render_model()));
    let draft = RwSignal::new(String::new());

    let _view = FloemChatView::render(
        surface,
        draft,
        FloemChatActions {
            on_attach: || {},
            on_remove_attachment: |_| {},
            on_new_chat: || {},
            on_history: || {},
            on_submit: |_| {},
            on_stop: || {},
            on_vendor_select: |_| {},
            on_control_select: |_, _| {},
        },
    );

    Ok(())
}

#[test]
fn panel_accepts_extension_slots_without_knowing_details() {
    let _view = FloemChatPanel::new()
        .header(FloemPanelSlot::new(text("header")))
        .thread(FloemPanelSlot::new(text("thread")))
        .extension(FloemPanelSlot::new(text("host output")))
        .composer(FloemPanelSlot::new(text("composer")))
        .render();
}

#[test]
fn standard_surface_uses_icon_text_and_vendor_overrides()
-> Result<(), katana_chat_ui::ChatSessionError> {
    let mut session = sample_session()?;
    session.override_icon(SvgIcon::new("send", SEND_ALT_ICON));
    session.set_text_catalog(
        TextCatalog::for_locale(ChatLocale::Ja).with_text(ChatTextKey::SendButton, "Run"),
    );
    session.set_vendor_ui_state(sample_vendor_state());

    let surface = ChatUiSurface::from_render_model(&session.render_model());

    assert!(surface.composer.send_icon.svg.contains("send-alt"));
    assert_eq!(surface.composer.send_label, "Run");
    assert_eq!(surface.vendor_ui.active_vendor_id, "claude-code");
    assert!(surface.vendor_ui.model_selector_visible);
    assert!(!surface.vendor_ui.mode_selector_visible);
    assert!(surface.vendor_ui.thinking_selector_visible);
    assert!(surface.vendor_ui.permission_mode_selector_visible);
    Ok(())
}

fn sample_session() -> Result<ChatSession, katana_chat_ui::ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.set_context_usage(ContextUsageSnapshot::new(920, 1000));
    session.draft_mut().set_text("hello");
    session
        .draft_mut()
        .add_attachment(Attachment::file(FileResource::new(
            "file:///tmp/note.md",
            "text/markdown",
            "# Note",
            6,
        )));
    session.submit_draft()?;
    Ok(session)
}

fn add_sample_output(session: &mut ChatSession) -> Result<(), katana_chat_ui::ChatSessionError> {
    let assistant_id = session.start_assistant_stream("sample")?;
    session.finish_assistant_message()?;
    session.add_output(
        assistant_id,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/generated.md",
            "text/markdown",
            "# generated",
        )),
    )?;
    Ok(())
}

fn sample_vendor_state() -> VendorUiState {
    VendorUiState::for_vendor("claude-code")
        .with_available_vendors(vec![
            "claude-code".to_string(),
            "codex-cli".to_string(),
            "github-copilot".to_string(),
            "opencode".to_string(),
        ])
        .with_models(vec!["claude-sonnet-4-6".to_string()], "claude-sonnet-4-6")
        .with_thinking(vec!["default".to_string(), "low".to_string()], "default")
        .with_permission_modes(vec!["default".to_string(), "auto".to_string()], "default")
}

const SEND_ALT_ICON: &str = r#"<svg data-kcu-icon="send-alt" viewBox="0 0 24 24"/>"#;
