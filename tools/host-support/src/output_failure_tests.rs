use katana_chat_ui::{
    ChatOutputKind, ChatSession, ChatUiAction, FileCandidateOutput, HostActionKind, OutputStatus,
    VendorUiState,
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
