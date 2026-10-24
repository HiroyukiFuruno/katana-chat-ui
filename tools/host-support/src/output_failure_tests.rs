use std::fs;

use katana_chat_ui::{
    ChatOutputKind, ChatSession, ChatUiAction, DiffCandidateOutput, FileCandidateOutput,
    HostActionKind, OutputStatus, VendorUiState,
};

use crate::{ChatHistoryStore, ChatHostState, ChatOutputExecutor};

#[test]
fn failed_undo_preserves_external_content_and_persists_reason()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let mut host = host(root.path())?;
    host.session_mut().draft_mut().set_text("create a file");
    let message = host.session_mut().submit_draft()?;
    let output = host.session_mut().add_output(
        message,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/a.md",
            "text/markdown",
            "ours",
        )),
    )?;
    host.dispatch(action(output, HostActionKind::CreateFile))?;
    std::fs::write(root.path().join("tmp/a.md"), "external edit")?;
    assert!(
        host.dispatch(action(output, HostActionKind::UndoChange))
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("tmp/a.md"))?,
        "external edit"
    );
    let restored = self::host(root.path())?;
    assert!(matches!(
        &restored.session().output(output)?.status,
        OutputStatus::Failed(reason) if reason.contains("content changed")
    ));
    Ok(())
}

fn action(output_id: u64, kind: HostActionKind) -> ChatUiAction {
    ChatUiAction::Output { output_id, kind }
}

#[test]
fn mismatched_output_action_cannot_write_a_file() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let mut host = host(root.path())?;
    let message = host.session_mut().start_assistant_stream("result")?;
    let output = host.session_mut().add_output(
        message,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/a.md",
            "text/markdown",
            "ours",
        )),
    )?;
    assert!(
        host.dispatch(action(output, HostActionKind::ApplyDiff))
            .is_err()
    );
    assert!(!root.path().join("tmp/a.md").exists());
    assert_eq!(host.session().output(output)?.status, OutputStatus::Ready);
    Ok(())
}

#[test]
fn apply_history_commit_failure_rolls_back_file_and_output_status()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let (mut host, file_id, _) = state_with_outputs(root.path())?;
    block_session_record(root.path(), &host.session_id)?;

    assert!(
        host.dispatch(action(file_id, HostActionKind::CreateFile))
            .is_err()
    );
    assert!(!root.path().join("tmp/generated.md").exists());
    assert_eq!(host.session().output(file_id)?.status, OutputStatus::Ready);
    Ok(())
}

#[test]
fn undo_history_commit_failure_restores_file_and_applied_status()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("tmp"))?;
    fs::write(root.path().join("tmp/edited.md"), "before")?;
    let (mut host, _, diff_id) = state_with_outputs(root.path())?;
    host.dispatch(action(diff_id, HostActionKind::ApplyDiff))?;
    block_session_record(root.path(), &host.session_id)?;

    assert!(
        host.dispatch(action(diff_id, HostActionKind::UndoChange))
            .is_err()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("tmp/edited.md"))?,
        "after"
    );
    assert_eq!(
        host.session().output(diff_id)?.status,
        OutputStatus::Applied
    );
    Ok(())
}

#[test]
fn apply_diff_and_undo_file_commit_history_after_external_change()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("tmp"))?;
    fs::write(root.path().join("tmp/edited.md"), "before")?;
    let (mut host, file_id, diff_id) = state_with_outputs(root.path())?;

    host.dispatch(action(diff_id, HostActionKind::ApplyDiff))?;
    assert_eq!(
        fs::read_to_string(root.path().join("tmp/edited.md"))?,
        "after"
    );
    assert_eq!(
        host.session().output(diff_id)?.status,
        OutputStatus::Applied
    );
    host.dispatch(action(file_id, HostActionKind::CreateFile))?;
    host.dispatch(action(file_id, HostActionKind::UndoChange))?;
    assert!(!root.path().join("tmp/generated.md").exists());
    assert_eq!(
        host.session().output(file_id)?.status,
        OutputStatus::Reverted
    );
    Ok(())
}

#[test]
fn prepare_failure_does_not_touch_external_output_or_live_session()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("tmp"))?;
    let (mut host, file_id, _) = state_with_outputs(root.path())?;
    let before = host.session().clone();
    fs::write(root.path().join("tmp/harness-katanagent"), "blocked")?;

    assert!(
        host.dispatch(action(file_id, HostActionKind::CreateFile))
            .is_err()
    );
    assert_eq!(host.session(), &before);
    assert!(!root.path().join("tmp/generated.md").exists());
    Ok(())
}

#[test]
fn apply_diff_history_failure_restores_file_and_session() -> Result<(), Box<dyn std::error::Error>>
{
    let root = tempfile::tempdir()?;
    fs::create_dir_all(root.path().join("tmp"))?;
    fs::write(root.path().join("tmp/edited.md"), "before")?;
    let (mut host, _, diff_id) = state_with_outputs(root.path())?;
    let before = host.session().clone();
    block_session_record(root.path(), &host.session_id)?;

    assert!(
        host.dispatch(action(diff_id, HostActionKind::ApplyDiff))
            .is_err()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("tmp/edited.md"))?,
        "before"
    );
    assert_eq!(host.session(), &before);
    Ok(())
}

#[test]
fn undo_file_history_failure_restores_file_and_session() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let (mut host, file_id, _) = state_with_outputs(root.path())?;
    host.dispatch(action(file_id, HostActionKind::CreateFile))?;
    let before = host.session().clone();
    block_session_record(root.path(), &host.session_id)?;

    assert!(
        host.dispatch(action(file_id, HostActionKind::UndoChange))
            .is_err()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("tmp/generated.md"))?,
        "generated"
    );
    assert_eq!(host.session(), &before);
    Ok(())
}

fn host(root: &std::path::Path) -> Result<ChatHostState, crate::ChatHostError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.set_vendor_ui_state(VendorUiState::for_vendor("katanagent"));
    ChatHostState::new(
        session,
        ChatHistoryStore::new(root.to_path_buf()),
        ChatOutputExecutor::new(root.to_path_buf()),
    )
}

fn state_with_outputs(
    root: &std::path::Path,
) -> Result<(ChatHostState, u64, u64), Box<dyn std::error::Error>> {
    let mut host = host(root)?;
    let message = host.session_mut().start_assistant_stream("response")?;
    let file_id = add_file_output(&mut host, message)?;
    let diff_id = add_diff_output(&mut host, message)?;
    Ok((host, file_id, diff_id))
}

fn add_file_output(
    host: &mut ChatHostState,
    message: u64,
) -> Result<u64, katana_chat_ui::ChatSessionError> {
    host.session_mut().add_output(
        message,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/generated.md",
            "text/markdown",
            "generated",
        )),
    )
}

fn add_diff_output(
    host: &mut ChatHostState,
    message: u64,
) -> Result<u64, katana_chat_ui::ChatSessionError> {
    host.session_mut().add_output(
        message,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/edited.md",
            "before",
            "after",
            "diff",
            "edit",
        )),
    )
}

fn block_session_record(root: &std::path::Path, session_id: &str) -> Result<(), std::io::Error> {
    let path = root
        .join("tmp/harness-katanagent/sessions")
        .join(format!("{session_id}.json"));
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| std::io::Error::other("session parent missing"))?,
    )?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_dir() => return Ok(()),
        Ok(_) => fs::remove_file(&path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs::create_dir(path)
}
