use std::{
    fs,
    path::{Path, PathBuf},
};

use katana_chat_ui::{
    ChatOutputKind, ChatSession, ChatUiAction, DiffCandidateOutput, FileCandidateOutput,
    VendorUiState,
};

use crate::{
    ChatHistoryStore, ChatHostEffect, ChatHostState, ChatOutputExecutor, ChatProviderPoll,
};

#[test]
fn typed_actions_persist_restore_and_select_history() -> Result<(), Box<dyn std::error::Error>> {
    let root = test_root("history");
    let mut state = state(&root)?;
    state
        .session_mut()
        .draft_mut()
        .set_text("persisted message");

    assert!(matches!(
        state.dispatch(ChatUiAction::Submit)?,
        ChatHostEffect::Submitted { .. }
    ));
    assert!(
        root.join("tmp/harness-katanagent/last-session.json")
            .is_file()
    );
    let restored = self::state(&root)?;
    assert_eq!(restored.session().render_model().messages.len(), 1);

    let mut state = restored;
    let session_id = first_history_session(&mut state)?;
    state.dispatch(ChatUiAction::NewChat)?;
    state.dispatch(ChatUiAction::SelectHistory { session_id })?;
    assert_eq!(state.session().render_model().messages.len(), 1);
    cleanup(root)?;
    Ok(())
}

#[test]
fn file_and_diff_actions_write_and_undo_only_under_tmp() -> Result<(), Box<dyn std::error::Error>> {
    let root = test_root("output");
    fs::create_dir_all(root.join("tmp"))?;
    fs::write(root.join("tmp/edited.md"), "before")?;
    let (mut state, file_id, diff_id) = state_with_outputs(&root)?;

    state.dispatch(output_action(
        file_id,
        katana_chat_ui::HostActionKind::CreateFile,
    ))?;
    assert_eq!(
        fs::read_to_string(root.join("tmp/generated.md"))?,
        "# generated"
    );
    state.dispatch(output_action(
        diff_id,
        katana_chat_ui::HostActionKind::ApplyDiff,
    ))?;
    assert_eq!(fs::read_to_string(root.join("tmp/edited.md"))?, "after");
    state.dispatch(output_action(
        diff_id,
        katana_chat_ui::HostActionKind::UndoChange,
    ))?;
    assert_eq!(fs::read_to_string(root.join("tmp/edited.md"))?, "before");
    state.dispatch(output_action(
        file_id,
        katana_chat_ui::HostActionKind::UndoChange,
    ))?;
    assert!(!root.join("tmp/generated.md").exists());
    cleanup(root)?;
    Ok(())
}

#[test]
fn provider_poll_keeps_unavailable_provider_unconfigured() -> Result<(), Box<dyn std::error::Error>>
{
    let root = test_root("poll");
    let mut state = state(&root)?;
    state.apply_provider_poll(ChatProviderPoll {
        state: VendorUiState::for_vendor("codex-cli"),
        unavailable_reason: Some("not installed".to_string()),
    });

    assert!(!state.session().render_model().input.can_submit);
    cleanup(root)?;
    Ok(())
}

#[test]
fn corrupt_history_does_not_block_host_restore_or_valid_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let root = test_root("corrupt-sibling");
    let store = ChatHistoryStore::new(root.clone());
    store.save("katanagent", "session-valid", ChatSession::new().snapshot())?;
    fs::write(
        root.join("tmp/harness-katanagent/sessions/bad.json"),
        "not json",
    )?;

    let mut state = state(&root)?;
    assert_eq!(state.session_id, "session-valid");
    assert!(state.take_history_diagnostics().is_some());
    state.dispatch(ChatUiAction::ToggleHistory)?;
    assert!(state.take_history_diagnostics().is_none());
    let session_id = state
        .surface()
        .history_panel
        .sessions
        .first()
        .ok_or("valid history is missing")?
        .session_id
        .clone();
    state.dispatch(ChatUiAction::SelectHistory { session_id })?;
    cleanup(root)?;
    Ok(())
}

fn state(root: &Path) -> Result<ChatHostState, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.set_vendor_ui_state(VendorUiState::for_vendor("katanagent"));
    Ok(ChatHostState::new(
        session,
        ChatHistoryStore::new(root.to_path_buf()),
        ChatOutputExecutor::new(root.to_path_buf()),
    )?)
}

fn first_history_session(state: &mut ChatHostState) -> Result<String, Box<dyn std::error::Error>> {
    state.dispatch(ChatUiAction::ToggleHistory)?;
    Ok(state
        .surface()
        .history_panel
        .sessions
        .first()
        .ok_or("saved history is missing")?
        .session_id
        .clone())
}

fn state_with_outputs(
    root: &Path,
) -> Result<(ChatHostState, u64, u64), Box<dyn std::error::Error>> {
    let mut state = state(root)?;
    let assistant_id = state.session_mut().start_assistant_stream("response")?;
    let file_id = state
        .session_mut()
        .add_output(assistant_id, file_output())?;
    let diff_id = state
        .session_mut()
        .add_output(assistant_id, diff_output())?;
    Ok((state, file_id, diff_id))
}

fn output_action(output_id: u64, kind: katana_chat_ui::HostActionKind) -> ChatUiAction {
    ChatUiAction::Output { output_id, kind }
}

fn file_output() -> ChatOutputKind {
    ChatOutputKind::FileCandidate(FileCandidateOutput::new(
        "tmp/generated.md",
        "text/markdown",
        "# generated",
    ))
}

fn diff_output() -> ChatOutputKind {
    ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
        "tmp/edited.md",
        "before",
        "after",
        "diff",
        "edit",
    ))
}

fn test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kcu-host-support-{label}-{}",
        ChatHistoryStore::new_session_id()
    ))
}

fn cleanup(path: PathBuf) -> Result<(), std::io::Error> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}
