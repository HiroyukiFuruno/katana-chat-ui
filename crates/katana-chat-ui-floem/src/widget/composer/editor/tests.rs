use super::commands::EditorCommandHandler;
use super::editor_text_for_reset;
use floem::{
    keyboard::Modifiers, prelude::*, views::editor::command::CommandExecuted,
    views::editor::keypress::press::KeyPress,
};
use katana_chat_ui::{ChatSession, ChatUiSurface};

const EDITOR_SOURCE: &str = include_str!("../editor.rs");

#[test]
fn editor_source_keeps_keypress_submit_wiring() {
    assert!(EDITOR_SOURCE.contains("text_editor_keys"));
    assert!(EDITOR_SOURCE.contains("EditorCommandHandler::handle_submit_keypress"));
}

#[test]
fn editor_source_syncs_draft_when_update_event_has_no_editor() {
    assert!(EDITOR_SOURCE.contains("let editor_for_update = editor.editor().clone();"));
    assert!(EDITOR_SOURCE.contains("editor.unwrap_or(fallback_editor)"));
}

#[test]
fn editor_uses_overlay_placeholder_outside_editor_text() {
    assert!(!EDITOR_SOURCE.contains(".placeholder(placeholder)"));
    assert!(EDITOR_SOURCE.contains("fn placeholder_overlay"));
    assert!(EDITOR_SOURCE.contains("placeholder_visible"));
    assert!(EDITOR_SOURCE.contains("stack((placeholder_overlay"));
    assert!(EDITOR_SOURCE.contains(".pointer_events_none()"));
}

#[test]
fn placeholder_overlay_is_visible_only_for_empty_draft() {
    assert!(super::placeholder_visible(""));
    assert!(!super::placeholder_visible("こんにちは"));
}

#[test]
fn editor_reset_does_not_use_placeholder_as_text() {
    let draft = RwSignal::new(String::new());

    assert_eq!(editor_text_for_reset("Ask anything".to_string(), draft), "");
}

#[test]
fn editor_reset_keeps_ime_live_draft_on_surface_refresh() {
    let draft = RwSignal::new("こんにちは".to_string());

    assert_eq!(
        editor_text_for_reset("Ask anything".to_string(), draft),
        "こんにちは"
    );
}

#[test]
fn keypress_command_enter_submits_even_before_editor_update_event() {
    let keypress = keypress("meta+enter");
    let surface = RwSignal::new(ready_surface());
    let draft = RwSignal::new("こんにちは".to_string());
    let submitted = RwSignal::new(Vec::<String>::new());

    let executed = EditorCommandHandler::handle_submit_keypress(
        &keypress,
        Modifiers::empty(),
        surface,
        draft,
        move |text| submitted.update(|it| it.push(text)),
    );

    assert_eq!(executed, CommandExecuted::Yes);
    assert_eq!(submitted.get_untracked(), vec!["こんにちは".to_string()]);
    assert_eq!(draft.get_untracked(), "");
}

#[test]
fn keypress_shift_command_enter_does_not_submit() {
    let keypress = keypress("meta+shift+enter");

    assert!(!EditorCommandHandler::is_submit_keypress(
        &keypress,
        Modifiers::empty()
    ));
}

#[test]
fn keypress_enter_without_command_does_not_submit() {
    let keypress = keypress("enter");

    assert!(!EditorCommandHandler::is_submit_keypress(
        &keypress,
        Modifiers::empty()
    ));
}

fn keypress(pattern: &str) -> KeyPress {
    let mut keypresses = KeyPress::parse(pattern);
    assert_eq!(keypresses.len(), 1);
    keypresses.remove(0)
}

fn ready_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    ChatUiSurface::from_render_model(&session.render_model())
}
