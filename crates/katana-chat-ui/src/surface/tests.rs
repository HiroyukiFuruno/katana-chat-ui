use super::ChatUiSurface;
use crate::{
    ChatOutputKind, ChatSession, ChatSessionError, ChatTextKey, ChatUiComposerInputKind,
    ChatUiMessageAlignment, ChatUiSurfaceProvider, FileCandidateOutput, MessageRole, MessageStatus,
    SvgIcon, TextCatalog, VendorUiState,
};

#[test]
fn surface_exposes_minimum_chat_ui() -> Result<(), ChatSessionError> {
    let surface = minimum_surface()?;

    assert_conversation_contract(&surface);
    assert_composer_contract(&surface);
    Ok(())
}

fn minimum_surface() -> Result<ChatUiSurface, ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("ollama");
    session.draft_mut().set_text("こんにちは");
    session.submit_draft()?;
    let assistant_id = session.start_assistant_stream("応答中")?;
    session.add_output(
        assistant_id,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/generated.md",
            "text/markdown",
            "# generated",
        )),
    )?;

    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn assert_conversation_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.messages.len(), 2);
    assert_eq!(surface.message_list.messages.len(), 2);
    assert_eq!(surface.messages[0].role, MessageRole::User);
    assert_eq!(
        surface.messages[0].alignment,
        ChatUiMessageAlignment::Trailing
    );
    assert_eq!(surface.messages[0].body, "こんにちは");
    assert_eq!(surface.messages[1].status, MessageStatus::Streaming);
    assert_eq!(
        surface.messages[1].alignment,
        ChatUiMessageAlignment::Leading
    );
    assert_eq!(surface.messages[1].outputs.len(), 1);
    assert_eq!(surface.message_list.messages[1].outputs.len(), 1);
}

fn assert_composer_contract(surface: &ChatUiSurface) {
    assert_eq!(
        surface.composer.input_kind,
        ChatUiComposerInputKind::ImeMultilineEditor
    );
    assert!(surface.composer.ime_enabled);
    assert!(surface.composer.stop_enabled);
}

#[test]
fn surface_reflects_runtime_icon_and_text_override() {
    let mut session = ChatSession::new();
    session.override_icon(SvgIcon::new("send", "<svg id=\"send-custom\"/>"));
    session.set_text_catalog(TextCatalog::english().with_text(ChatTextKey::SendButton, "Submit"));

    let surface = ChatUiSurface::from_render_model(&session.render_model());

    assert_eq!(surface.composer.send_icon.svg, "<svg id=\"send-custom\"/>");
    assert_eq!(surface.composer.send.icon.svg, "<svg id=\"send-custom\"/>");
    assert_eq!(surface.composer.send_label, "Submit");
}

#[test]
fn surface_localizes_chat_ui_labels() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("ollama");
    session.set_text_catalog(TextCatalog::for_locale(crate::ChatLocale::Ja));
    session.draft_mut().set_text("日本語入力");
    session.submit_draft()?;

    let surface = ChatUiSurface::from_render_model(&session.render_model());

    assert_eq!(surface.messages[0].role_label, "ユーザー");
    assert_eq!(surface.composer.placeholder, "メッセージを入力");
    assert_eq!(surface.output_handoff.label, "出力");
    Ok(())
}

#[test]
fn surface_exposes_vendor_specific_affordances() {
    let surface = ollama_vendor_surface();

    assert_eq!(surface.vendor_ui.active_vendor_id, "ollama");
    assert!(
        surface
            .vendor_bar
            .vendor_options
            .iter()
            .any(|it| it.id == "claude-code")
    );
    assert_eq!(
        surface
            .vendor_controls
            .endpoint
            .as_ref()
            .and_then(|it| it.selected_value.clone()),
        Some("http://localhost:11434".to_string())
    );
    assert!(surface.vendor_ui.model_selector_visible);
    assert!(surface.vendor_ui.thinking_selector_visible);
    assert!(!surface.vendor_ui.permission_mode_selector_visible);
    assert_vendor_bar_contract(&surface);
    assert!(surface.vendor_controls.usage_visible);
    assert!(!surface.vendor_controls.account_usage_visible);
}

fn ollama_vendor_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("ollama")
            .with_available_vendors(vec!["ollama".to_string(), "claude-code".to_string()])
            .with_endpoint("http://localhost:11434")
            .with_models(vec!["llama3".to_string()], "llama3")
            .with_thinking(vec!["false".to_string(), "low".to_string()], "low"),
    );

    ChatUiSurface::from_render_model(&session.render_model())
}

fn assert_vendor_bar_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.vendor_bar.active_vendor_label, "Ollama local");
    assert_eq!(surface.vendor_bar.vendor_selector_label, "Provider");
    assert!(
        surface
            .vendor_bar
            .controls
            .iter()
            .any(|it| it.key == "endpoint")
    );
    assert!(
        surface
            .vendor_bar
            .controls
            .iter()
            .any(|it| it.key == "model" && it.options == vec!["llama3".to_string()])
    );
}

#[test]
fn session_exposes_surface_through_interface() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("ollama");
    session.draft_mut().set_text("interface");
    session.submit_draft()?;

    let surface = session.chat_ui_surface();

    assert_eq!(surface.message_list.messages.len(), 1);
    assert!(surface.composer.ime_enabled);
    Ok(())
}
