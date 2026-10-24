use std::{
    fs,
    path::{Path, PathBuf},
};

use katana_chat_ui::{
    ChatOutputKind, ChatSession, ChatUiAction, CodeOutput, DiffCandidateOutput,
    FileCandidateOutput, HostActionKind, OutputStatus, PermissionRequestOutput, TextOutput,
    ToolResultOutput, VendorUiState,
};

use crate::{
    ChatHistoryStore, ChatHostEffect, ChatHostError, ChatHostState, ChatOutputExecutor,
    ChatProviderPoll,
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

    dispatch_visible_action(&mut state, file_id, HostActionKind::CreateFile)?;
    assert_eq!(
        fs::read_to_string(root.join("tmp/generated.md"))?,
        "# generated"
    );
    dispatch_visible_action(&mut state, diff_id, HostActionKind::ApplyDiff)?;
    assert_eq!(fs::read_to_string(root.join("tmp/edited.md"))?, "after");
    dispatch_visible_action(&mut state, diff_id, HostActionKind::UndoChange)?;
    assert_eq!(fs::read_to_string(root.join("tmp/edited.md"))?, "before");
    dispatch_visible_action(&mut state, file_id, HostActionKind::UndoChange)?;
    assert!(!root.join("tmp/generated.md").exists());
    cleanup(root)?;
    Ok(())
}

#[test]
fn standard_host_surfaces_filter_unimplemented_output_actions_consistently()
-> Result<(), Box<dyn std::error::Error>> {
    let root = test_root("standard-actions");
    let mut state = state(&root)?;
    let assistant_id = state.session_mut().start_assistant_stream("response")?;
    add_status_matrix(&mut state, assistant_id)?;
    add_unimplemented_outputs(&mut state, assistant_id)?;
    let before = state.session().snapshot();
    assert_public_preview_intents(&before);
    let original =
        katana_chat_ui::ChatUiSurface::from_render_model(&state.session().render_model());
    let surface = state.surface();
    assert_projection_actions(&surface, &original)?;
    assert_projection_payloads(&surface, &original)?;
    assert_eq!(state.session().snapshot(), before);
    cleanup(root)?;
    Ok(())
}

#[test]
fn standard_host_projection_filter_covers_populated_message_views()
-> Result<(), Box<dyn std::error::Error>> {
    let root = test_root("populated-output-views");
    let mut state = state(&root)?;
    let assistant_id = state.session_mut().start_assistant_stream("response")?;
    add_status_matrix(&mut state, assistant_id)?;
    add_unimplemented_outputs(&mut state, assistant_id)?;
    let mut surface =
        katana_chat_ui::ChatUiSurface::from_render_model(&state.session().render_model());
    let outputs = surface.output_handoff.outputs.clone();
    surface
        .messages
        .last_mut()
        .ok_or("message missing")?
        .outputs = outputs.clone();
    surface
        .message_list
        .messages
        .last_mut()
        .ok_or("message list missing")?
        .outputs = outputs;
    let original = surface.clone();
    crate::surface::restrict_output_actions(&mut surface);
    assert_projection_actions(&surface, &original)?;
    assert_projection_payloads(&surface, &original)?;
    cleanup(root)?;
    Ok(())
}

fn add_status_matrix(
    state: &mut ChatHostState,
    source_message_id: u64,
) -> Result<(), ChatHostError> {
    for status in [
        OutputStatus::Ready,
        OutputStatus::PendingHost,
        OutputStatus::Applied,
        OutputStatus::Reverted,
        OutputStatus::Rejected,
        OutputStatus::Failed("failed".to_string()),
    ] {
        let file_id = state
            .session_mut()
            .add_output(source_message_id, file_output())?;
        state
            .session_mut()
            .set_output_status(file_id, status.clone())?;
        let diff_id = state
            .session_mut()
            .add_output(source_message_id, diff_output())?;
        state.session_mut().set_output_status(diff_id, status)?;
    }
    Ok(())
}

fn add_unimplemented_outputs(
    state: &mut ChatHostState,
    source_message_id: u64,
) -> Result<(), ChatHostError> {
    for kind in [
        ChatOutputKind::Text(TextOutput::new("text")),
        ChatOutputKind::Code(CodeOutput::new(None, "code")),
        ChatOutputKind::ToolResult(ToolResultOutput::new("tool", "result")),
        ChatOutputKind::PermissionRequest(PermissionRequestOutput::new("allow", "reason")),
    ] {
        state.session_mut().add_output(source_message_id, kind)?;
    }
    Ok(())
}

fn assert_projection_actions(
    surface: &katana_chat_ui::ChatUiSurface,
    original: &katana_chat_ui::ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    for (outputs, source) in output_projections(surface)?
        .into_iter()
        .zip(output_projections(original)?)
    {
        let expected = source.iter().map(expected_actions).collect::<Vec<_>>();
        assert_action_projection(outputs, &expected);
    }
    Ok(())
}

fn output_projections(
    surface: &katana_chat_ui::ChatUiSurface,
) -> Result<[&[katana_chat_ui::ChatUiOutputSurface]; 3], Box<dyn std::error::Error>> {
    let message = surface.messages.last().ok_or("message surface missing")?;
    let message_list = surface
        .message_list
        .messages
        .last()
        .ok_or("message list missing")?;
    Ok([
        &message.outputs,
        &message_list.outputs,
        &surface.output_handoff.outputs,
    ])
}

fn assert_action_projection(
    outputs: &[katana_chat_ui::ChatUiOutputSurface],
    expected: &[Vec<HostActionKind>],
) {
    let actual = outputs
        .iter()
        .map(|output| {
            output
                .actions
                .iter()
                .map(|action| action.kind)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn assert_projection_payloads(
    surface: &katana_chat_ui::ChatUiSurface,
    original: &katana_chat_ui::ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    for (outputs, source) in output_projections(surface)?
        .into_iter()
        .zip(output_projections(original)?)
    {
        assert_payload_projection(outputs, &expected_payloads(source));
    }
    Ok(())
}

fn expected_payloads(
    outputs: &[katana_chat_ui::ChatUiOutputSurface],
) -> Vec<(u64, u64, ChatOutputKind, OutputStatus)> {
    outputs
        .iter()
        .map(|output| {
            (
                output.id,
                output.source_message_id,
                output.kind.clone(),
                output.status.clone(),
            )
        })
        .collect()
}

fn assert_payload_projection(
    outputs: &[katana_chat_ui::ChatUiOutputSurface],
    expected: &[(u64, u64, ChatOutputKind, OutputStatus)],
) {
    let actual = outputs
        .iter()
        .map(|output| {
            (
                output.id,
                output.source_message_id,
                output.kind.clone(),
                output.status.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn expected_actions(output: &katana_chat_ui::ChatUiOutputSurface) -> Vec<HostActionKind> {
    match &output.kind {
        ChatOutputKind::FileCandidate(_) | ChatOutputKind::DiffCandidate(_) => {
            candidate_actions(&output.kind, &output.status)
        }
        _ => Vec::new(),
    }
}

fn candidate_actions(kind: &ChatOutputKind, status: &OutputStatus) -> Vec<HostActionKind> {
    match (kind, status) {
        (ChatOutputKind::FileCandidate(_), OutputStatus::Applied)
        | (ChatOutputKind::DiffCandidate(_), OutputStatus::Applied) => {
            vec![HostActionKind::UndoChange]
        }
        (ChatOutputKind::FileCandidate(_), OutputStatus::Reverted)
        | (ChatOutputKind::FileCandidate(_), OutputStatus::Rejected)
        | (ChatOutputKind::FileCandidate(_), OutputStatus::Failed(_))
        | (ChatOutputKind::DiffCandidate(_), OutputStatus::Reverted)
        | (ChatOutputKind::DiffCandidate(_), OutputStatus::Rejected)
        | (ChatOutputKind::DiffCandidate(_), OutputStatus::Failed(_)) => Vec::new(),
        (ChatOutputKind::FileCandidate(_), OutputStatus::Ready)
        | (ChatOutputKind::FileCandidate(_), OutputStatus::PendingHost) => {
            vec![HostActionKind::CreateFile]
        }
        (ChatOutputKind::DiffCandidate(_), OutputStatus::Ready)
        | (ChatOutputKind::DiffCandidate(_), OutputStatus::PendingHost) => {
            vec![HostActionKind::ApplyDiff]
        }
        _ => Vec::new(),
    }
}

fn assert_public_preview_intents(snapshot: &katana_chat_ui::ChatSessionSnapshot) {
    for output in &snapshot.outputs {
        if matches!(
            output.kind,
            ChatOutputKind::FileCandidate(_) | ChatOutputKind::DiffCandidate(_)
        ) {
            assert!(
                output
                    .host_actions()
                    .iter()
                    .any(|action| action.kind == HostActionKind::OpenPreview)
            );
        }
    }
}

fn dispatch_visible_action(
    state: &mut ChatHostState,
    output_id: u64,
    kind: HostActionKind,
) -> Result<(), ChatHostError> {
    let surface = state.surface();
    assert!(surface.output_handoff.outputs.iter().any(|output| {
        output.id == output_id && output.actions.iter().any(|action| action.kind == kind)
    }));
    state.dispatch(output_action(output_id, kind))?;
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
fn busy_submit_preserves_host_snapshot_and_draft() -> Result<(), Box<dyn std::error::Error>> {
    let root = test_root("busy-submit");
    let mut state = state(&root)?;
    state.session_mut().start_assistant_stream("partial")?;
    state.session_mut().draft_mut().set_text("keep this draft");
    let before = state.session().snapshot();
    let error = match state.dispatch(ChatUiAction::Submit) {
        Err(error) => error,
        Ok(effect) => return Err(format!("busy submit unexpectedly succeeded: {effect:?}").into()),
    };
    assert!(
        matches!(error, ChatHostError::Provider(message) if message.contains("active provider"))
    );
    assert_eq!(state.session().snapshot(), before);
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
