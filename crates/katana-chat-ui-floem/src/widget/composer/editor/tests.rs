use super::commands::EditorCommandHandler;
use floem::{
    keyboard::Modifiers,
    prelude::*,
    views::editor::command::{Command, CommandExecuted},
};
use floem_editor_core::command::EditCommand;
use katana_chat_ui::{ChatSession, ChatUiSurface};

const EDITOR_SOURCE: &str = include_str!("../editor.rs");

#[test]
fn command_enter_submits_with_floem_default_command() {
    let command = Command::Edit(EditCommand::NewLineBelow);

    assert!(EditorCommandHandler::is_submit_command(
        &command,
        Modifiers::META
    ));
}

#[test]
fn enter_without_command_keeps_editor_newline() {
    let command = Command::Edit(EditCommand::InsertNewLine);

    assert!(!EditorCommandHandler::is_submit_command(
        &command,
        Modifiers::empty()
    ));
}

#[test]
fn command_shift_enter_does_not_submit() {
    let command = Command::Edit(EditCommand::NewLineAbove);

    assert!(!EditorCommandHandler::is_submit_command(
        &command,
        Modifiers::META | Modifiers::SHIFT
    ));
}

#[test]
fn submit_command_consumes_editor_command_once() {
    let command = Command::Edit(EditCommand::NewLineBelow);
    let executed = if EditorCommandHandler::is_submit_command(&command, Modifiers::META) {
        CommandExecuted::Yes
    } else {
        CommandExecuted::No
    };

    assert_eq!(executed, CommandExecuted::Yes);
}

#[test]
fn editor_source_keeps_pre_command_submit_wiring() {
    assert!(EDITOR_SOURCE.contains(".pre_command"));
    assert!(EDITOR_SOURCE.contains("EditorCommandHandler::handle_pre_command"));
}

#[test]
fn pre_command_submits_draft_once() {
    let command = Command::Edit(EditCommand::NewLineBelow);
    let surface = RwSignal::new(ready_surface());
    let draft = RwSignal::new("こんにちは".to_string());
    let submitted = RwSignal::new(Vec::<String>::new());

    let executed = EditorCommandHandler::handle_pre_command(
        &command,
        Modifiers::META,
        surface,
        draft,
        move |text| {
            submitted.update(|it| it.push(text));
        },
    );

    assert_eq!(executed, CommandExecuted::Yes);
    assert_eq!(submitted.get_untracked(), vec!["こんにちは".to_string()]);
}

#[test]
fn pre_command_leaves_plain_enter_to_editor() {
    let command = Command::Edit(EditCommand::InsertNewLine);
    let surface = RwSignal::new(ready_surface());
    let draft = RwSignal::new("こんにちは".to_string());
    let submitted = RwSignal::new(Vec::<String>::new());

    let executed = EditorCommandHandler::handle_pre_command(
        &command,
        Modifiers::empty(),
        surface,
        draft,
        move |text| {
            submitted.update(|it| it.push(text));
        },
    );

    assert_eq!(executed, CommandExecuted::No);
    assert!(submitted.get_untracked().is_empty());
}

fn ready_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("ollama");
    ChatUiSurface::from_render_model(&session.render_model())
}
