use super::{ChatSession, ChatSessionError};
use crate::{ChatLocale, ChatRenderModel, ChatSessionSnapshot};
use crate::{
    ChatOutputKind, CommandLaunchEntry, CommandLaunchKind, DiffCandidateOutput,
    FileCandidateOutput, HostActionKind, MessageAlignment, MessageRole, MessageStatus,
    OutputStatus, PermissionRequestOutput, ProviderConnectionState, VendorUiOption,
    VendorUiPermissionRequest, VendorUiState,
};

#[test]
fn render_model_exposes_turn_statuses_and_provider_state() -> Result<(), ChatSessionError> {
    let disabled = missing_provider_model();

    assert!(!disabled.input.can_submit);
    assert!(matches!(
        disabled.provider,
        ProviderConnectionState::Missing(_)
    ));

    let model = streaming_model()?;

    assert_eq!(model.messages.len(), 2);
    assert_eq!(model.messages[0].role, MessageRole::User);
    assert_eq!(
        model.messages[0].visual.alignment,
        MessageAlignment::Trailing
    );
    assert_eq!(model.messages[1].role, MessageRole::Assistant);
    assert_eq!(model.messages[1].status, MessageStatus::Complete);
    Ok(())
}

fn missing_provider_model() -> ChatRenderModel {
    let mut session = ChatSession::new();
    session.set_provider_missing("provider is not configured");
    session.draft_mut().set_text("hello");
    session.render_model()
}

fn streaming_model() -> Result<ChatRenderModel, ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("hello");
    session.submit_draft()?;
    session.start_assistant_stream("working")?;
    session.append_assistant_chunk(" done")?;
    session.finish_assistant_message()?;
    Ok(session.render_model())
}

#[test]
fn render_model_exposes_output_handoff_actions() -> Result<(), ChatSessionError> {
    let model = output_handoff_model()?;

    assert_eq!(model.outputs.len(), 3);
    assert_eq!(model.outputs[0].status, OutputStatus::PendingHost);
    assert_output_action(&model, 0, HostActionKind::CreateFile);
    assert_output_action(&model, 1, HostActionKind::ApplyDiff);
    assert_output_action(&model, 2, HostActionKind::Approve);
    Ok(())
}

fn output_handoff_model() -> Result<ChatRenderModel, ChatSessionError> {
    let (mut session, assistant_id) = completed_assistant_session()?;
    let file_output_id = add_file_output(&mut session, assistant_id)?;
    add_diff_output(&mut session, assistant_id)?;
    add_permission_output(&mut session, assistant_id)?;
    session.set_output_status(file_output_id, OutputStatus::PendingHost)?;
    Ok(session.render_model())
}

fn completed_assistant_session() -> Result<(ChatSession, u64), ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("generate output");
    session.submit_draft()?;
    let assistant_id = session.start_assistant_stream("done")?;
    session.finish_assistant_message()?;
    Ok((session, assistant_id))
}

fn add_file_output(session: &mut ChatSession, assistant_id: u64) -> Result<u64, ChatSessionError> {
    session.add_output(
        assistant_id,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "src/generated.rs",
            "text/rust",
            "pub struct Generated;",
        )),
    )
}

fn add_diff_output(session: &mut ChatSession, assistant_id: u64) -> Result<u64, ChatSessionError> {
    session.add_output(
        assistant_id,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "src/lib.rs",
            "hello",
            "hello kcu",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
            "src/lib.rs を更新",
        )),
    )
}

fn add_permission_output(
    session: &mut ChatSession,
    assistant_id: u64,
) -> Result<u64, ChatSessionError> {
    session.add_output(
        assistant_id,
        ChatOutputKind::PermissionRequest(PermissionRequestOutput::new(
            "run tests",
            "Agent response requested host-side verification",
        )),
    )
}

fn assert_output_action(model: &ChatRenderModel, output_index: usize, expected: HostActionKind) {
    assert!(
        model.outputs[output_index]
            .actions
            .iter()
            .any(|it| it.kind == expected)
    );
}

#[test]
fn render_model_uses_session_locale_and_text_overrides() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_locale(ChatLocale::Ja);
    let override_result = session.apply_text_override_json(r#"{"send_button":"Run"}"#);
    assert!(override_result.is_ok());

    let texts = session.render_model().texts;

    assert_eq!(texts.locale, "ja");
    assert_eq!(texts.send_button, "Run");
    assert_eq!(texts.stop_button, "停止");
    Ok(())
}

#[test]
fn render_model_exposes_conversation_title_and_provider_icon() {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("claude-code")
            .with_available_vendors(vec!["claude-code".to_string()]),
    );
    session.set_title("Generated chat title");

    let model = session.render_model();

    assert_eq!(model.title, "Generated chat title");
    assert_eq!(model.icons.provider.asset_id, "provider:claude-code");
}

#[test]
fn render_model_exposes_slash_launcher_without_mutating_draft() {
    let mut session = ChatSession::new();
    session.set_command_entries(vec![CommandLaunchEntry::new(
        "review",
        "レビュー",
        CommandLaunchKind::Workflow,
    )]);
    session.draft_mut().set_text("/レ");

    let model = session.render_model();

    assert_eq!(model.input.text, "/レ");
    assert!(model.input.slash_launcher.visible);
    assert_eq!(model.input.slash_launcher.entries[0].id, "review");
}

#[test]
fn snapshot_roundtrip_restores_messages_outputs_and_provider_state() -> Result<(), ChatSessionError>
{
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("katanagent")
            .with_models(vec!["ollama:gemma4:e4b".to_string()], "ollama:gemma4:e4b"),
    );
    session.draft_mut().set_text("sample を作成");
    session.submit_draft()?;
    let assistant_id = session.start_assistant_stream("作成しました")?;
    session.finish_assistant_message()?;
    let output_id = add_file_output(&mut session, assistant_id)?;
    session.set_output_status(output_id, OutputStatus::Applied)?;
    session.draft_mut().set_text("next draft");
    let snapshot = session.snapshot();

    let mut restored = ChatSession::new();
    restored.restore_snapshot(snapshot.clone());
    let model = restored.render_model();

    assert_eq!(restored.snapshot(), snapshot);
    assert_eq!(model.messages.len(), 2);
    assert_eq!(model.outputs.len(), 1);
    assert_eq!(model.vendor_ui.active_vendor_id, "katanagent");
    assert_eq!(model.outputs[0].status, OutputStatus::Applied);
    assert_eq!(model.input.text, "next draft");
    Ok(())
}

#[test]
fn restore_snapshot_interrupts_transport_state_without_stale_permission()
-> Result<(), ChatSessionError> {
    let snapshot = interrupted_snapshot()?;
    let mut restored = ChatSession::new();

    restored.restore_snapshot(snapshot);

    let model = restored.render_model();
    assert_restored_messages_are_terminal(&model);
    assert_eq!(model.vendor_ui.pending_permission, None);
    assert!(!model.input.can_cancel);
    assert_eq!(model.input.text, "unsent draft");
    Ok(())
}

fn interrupted_snapshot() -> Result<ChatSessionSnapshot, ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("resume later");
    session.submit_draft()?;
    session.start_assistant_stream_with_thinking(
        "partial response",
        crate::ThinkingLog::running("working", vec!["partial".into()]),
    )?;
    session.set_assistant_activity(crate::AgentActivityKind::Generating)?;
    let mut state = VendorUiState::for_vendor("claude-code");
    state.pending_permission = Some(VendorUiPermissionRequest {
        request_id: 7,
        tool_call_id: Some("tool-7".into()),
        title: Some("Run command".into()),
        details: vec!["permission was requested before restart".into()],
        options: vec![VendorUiOption {
            id: "approve".into(),
            label: "Approve".into(),
        }],
    });
    session.set_vendor_ui_state(state);
    session.draft_mut().set_text("unsent draft");
    let mut snapshot = session.snapshot();
    snapshot.messages[0].status = MessageStatus::Sending;
    Ok(snapshot)
}

fn assert_restored_messages_are_terminal(model: &crate::ChatRenderModel) {
    assert_eq!(model.messages.len(), 2);
    assert_eq!(model.messages[0].id, 1);
    assert_eq!(model.messages[1].id, 2);
    for message in &model.messages {
        assert_eq!(
            message.status,
            MessageStatus::Error("interrupted while restoring session".into())
        );
        assert_eq!(message.activity, None);
    }
    assert_eq!(
        model.messages[0].blocks,
        crate::MarkdownSubset::parse("resume later").blocks
    );
    assert_eq!(
        model.messages[1].blocks,
        crate::MarkdownSubset::parse("partial response").blocks
    );
    assert!(
        model.messages[1]
            .thinking
            .as_ref()
            .is_some_and(|thinking| thinking.completed && !thinking.expanded)
    );
}
