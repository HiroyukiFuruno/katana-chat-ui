use super::composer::{ComposerDraftSubmitter, ComposerEditorState, ComposerSubmitGate};
use floem::prelude::*;
use katana_chat_ui::{ChatSession, ChatUiComposerSurface, ChatUiSurface};

#[test]
fn editor_initial_text_keeps_live_draft_after_surface_refresh() {
    let composer = composer_surface("");
    let draft = RwSignal::new("こんにちは".to_string());

    assert_eq!(
        ComposerEditorState::initial_text(&composer, draft),
        "こんにちは"
    );
}

#[test]
fn editor_initial_text_uses_surface_text_when_draft_is_empty() {
    let composer = composer_surface("surface draft");
    let draft = RwSignal::new(String::new());

    assert_eq!(
        ComposerEditorState::initial_text(&composer, draft),
        "surface draft"
    );
}

#[test]
fn submitter_sends_non_empty_live_draft() {
    let draft = RwSignal::new("こんにちは".to_string());
    let submitted = RwSignal::new(String::new());

    ComposerDraftSubmitter::submit(draft, move |text| submitted.set(text));

    assert_eq!(submitted.get_untracked(), "こんにちは");
}

#[test]
fn submitter_ignores_blank_draft() {
    let draft = RwSignal::new("   ".to_string());
    let submitted = RwSignal::new(false);

    ComposerDraftSubmitter::submit(draft, move |_| submitted.set(true));

    assert!(!submitted.get_untracked());
}

#[test]
fn submit_gate_allows_attachment_only_when_provider_is_ready() {
    let mut composer = composer_surface("");
    composer
        .attachments
        .push(katana_chat_ui::Attachment::text("sample.md", "# sample"));

    assert!(ComposerSubmitGate::can_submit(&composer, ""));
}

#[test]
fn submit_gate_rejects_text_when_provider_is_missing() {
    let mut composer = composer_surface("");
    composer.send_available = false;

    assert!(!ComposerSubmitGate::can_submit(&composer, "こんにちは"));
}

#[test]
fn submit_gate_rejects_submit_while_stop_is_active() {
    let mut composer = composer_surface("");
    composer.stop_enabled = true;

    assert!(!ComposerSubmitGate::can_submit(&composer, "こんにちは"));
}

fn composer_surface(text: &str) -> ChatUiComposerSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("ollama");
    session.draft_mut().set_text(text);
    ChatUiSurface::from_render_model(&session.render_model()).composer
}
