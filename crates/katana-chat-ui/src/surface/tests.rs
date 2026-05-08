use super::ChatUiSurface;
use crate::{
    ChatOutputKind, ChatSession, ChatSessionError, ChatTextKey, ChatUiComposerInputKind,
    ChatUiMessageAlignment, ChatUiSurfaceProvider, ContextUsageSnapshot, FileCandidateOutput,
    MessageRole, MessageStatus, SvgIcon, TextCatalog,
};

#[test]
fn surface_exposes_minimum_chat_ui() -> Result<(), ChatSessionError> {
    let surface = minimum_surface()?;

    assert_conversation_contract(&surface);
    assert_chrome_contract(&surface);
    assert_composer_contract(&surface);
    Ok(())
}

fn minimum_surface() -> Result<ChatUiSurface, ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.set_context_usage(ContextUsageSnapshot::new(750, 1000));
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
    assert_user_message_contract(surface);
    assert_assistant_message_contract(surface);
    assert_output_handoff_contract(surface);
}

fn assert_user_message_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.messages[0].role, MessageRole::User);
    assert_eq!(
        surface.messages[0].alignment,
        ChatUiMessageAlignment::Trailing
    );
    assert_eq!(surface.messages[0].body, "こんにちは");
}

fn assert_assistant_message_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.messages[1].status, MessageStatus::Streaming);
    assert_eq!(
        surface.messages[1].alignment,
        ChatUiMessageAlignment::Leading
    );
    assert!(surface.messages[1].outputs.is_empty());
    assert!(surface.message_list.messages[1].outputs.is_empty());
}

fn assert_output_handoff_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.output_handoff.outputs.len(), 1);
}

fn assert_chrome_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.chrome.new_chat.id, "new-chat");
    assert_eq!(surface.chrome.history.id, "history");
}

fn assert_composer_contract(surface: &ChatUiSurface) {
    assert_eq!(
        surface.composer.input_kind,
        ChatUiComposerInputKind::ImeMultilineEditor
    );
    assert!(surface.composer.ime_enabled);
    assert!(surface.composer.stop_enabled);
    assert_eq!(surface.usage.used_tokens, 750);
    assert_eq!(surface.usage.max_tokens, 1000);
    assert_eq!(surface.usage.context_percentage, 75);
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
    session.set_provider_configured("Claude Code");
    session.set_text_catalog(TextCatalog::for_locale(crate::ChatLocale::Ja));
    session.draft_mut().set_text("日本語入力");
    session.submit_draft()?;

    let surface = ChatUiSurface::from_render_model(&session.render_model());

    assert_eq!(surface.messages[0].role_label, "ユーザー");
    assert_eq!(surface.composer.placeholder, "メッセージを入力");
    assert_eq!(surface.chrome.new_chat.label, "新しい会話");
    assert_eq!(surface.chrome.history.label, "履歴");
    assert_eq!(surface.output_handoff.label, "出力");
    Ok(())
}

#[test]
fn session_exposes_surface_through_interface() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("interface");
    session.submit_draft()?;

    let surface = session.chat_ui_surface();

    assert_eq!(surface.message_list.messages.len(), 1);
    assert!(surface.composer.ime_enabled);
    Ok(())
}
