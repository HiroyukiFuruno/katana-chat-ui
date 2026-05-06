use super::{ChatSession, ChatSessionError};
use crate::{ChatLocale, ChatRenderModel};
use crate::{
    ChatOutputKind, DiffCandidateOutput, FileCandidateOutput, HostActionKind, MessageAlignment,
    MessageRole, MessageStatus, OutputStatus, PermissionRequestOutput, ProviderConnectionState,
    VendorUiState,
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
    session.set_provider_configured("ollama");
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
    session.set_provider_configured("ollama");
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
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
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
            "Ollama response requested host-side verification",
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
    session.set_provider_configured("ollama");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("ollama").with_available_vendors(vec!["ollama".to_string()]),
    );
    session.set_title("Generated chat title");

    let model = session.render_model();

    assert_eq!(model.title, "Generated chat title");
    assert_eq!(model.icons.provider.asset_id, "provider:ollama");
}
