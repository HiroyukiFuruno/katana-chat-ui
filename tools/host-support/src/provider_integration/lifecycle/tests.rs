use super::{complete, config_rejected, connecting, fail, ready};
use crate::{AcpProviderModel, ChatHistoryStore, ChatHostState, ChatOutputExecutor};
use katana_acp_client::protocol::{AcpPermissionContext, AcpPermissionOption};
use katana_chat_ui::{ChatSession, ProviderConnectionState, VendorUiState};

use super::super::permission;

#[test]
fn completion_clears_a_pending_permission() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = host()?;
    project(&mut host);
    let mut streaming = false;

    complete(&mut host, &mut streaming)?;

    assert!(
        host.session()
            .vendor_ui_state()
            .pending_permission
            .is_none()
    );
    Ok(())
}

#[test]
fn failure_clears_permission_and_repairs_streaming_state() -> Result<(), Box<dyn std::error::Error>>
{
    let mut host = host()?;
    host.session_mut().start_assistant_stream(String::new())?;
    project(&mut host);
    let mut streaming = true;

    fail(&mut host, &mut streaming, "prompt rejected".to_string())?;

    assert!(!streaming);
    assert!(
        host.session()
            .vendor_ui_state()
            .pending_permission
            .is_none()
    );
    assert!(!host.session().render_model().input.can_submit);
    Ok(())
}

#[test]
fn rejected_model_change_preserves_ready_permission_and_streaming_state()
-> Result<(), Box<dyn std::error::Error>> {
    let mut host = active_host_with_permission()?;

    let error = match config_rejected(&mut host, "Invalid params".into()) {
        Err(error) => error,
        Ok(()) => return Err("configuration rejection must be surfaced".into()),
    };

    assert!(error.to_string().contains("Invalid params"));
    assert_active_state(&host);

    host.session_mut()
        .append_assistant_chunk("continued response")?;
    ready(&mut host, "codex-cli", &[model()], false);
    assert_active_state(&host);
    Ok(())
}

#[test]
fn configuration_rejection_persists_the_preceding_confirmed_model()
-> Result<(), Box<dyn std::error::Error>> {
    let mut host = active_host_with_permission()?;
    let mut vendor = host.session().vendor_ui_state().clone();
    vendor.selected_model = Some("previous".into());
    host.session_mut().set_vendor_ui_state(vendor);
    host.persist()?;
    ready(&mut host, "codex-cli", &[model()], false);
    let before = host.session().clone();

    let result = config_rejected(&mut host, "later selection rejected".into());

    assert!(
        matches!(result, Err(crate::ChatHostError::Provider(reason)) if reason.contains("later selection rejected"))
    );
    let saved = host
        .history
        .load(&host.session_id)?
        .ok_or("missing history")?;
    assert_eq!(
        saved.snapshot.vendor_state.selected_model.as_deref(),
        Some("model-a")
    );
    assert_eq!(host.session(), &before);
    assert_active_state(&host);
    Ok(())
}

#[test]
fn configuration_rejection_reports_history_write_failure() -> Result<(), Box<dyn std::error::Error>>
{
    let root = tempfile::tempdir()?;
    let blocked = root.path().join("history-file");
    std::fs::write(&blocked, "not a directory")?;
    let mut host = active_host_with_permission()?;
    host.history = ChatHistoryStore::new(blocked);
    let before = host.session().clone();

    let result = config_rejected(&mut host, "selection rejected".into());

    assert!(matches!(result, Err(crate::ChatHostError::History(_))));
    assert_eq!(host.session(), &before);
    Ok(())
}

#[test]
fn initialization_clears_a_previous_permission_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = host()?;
    project(&mut host);

    ready(&mut host, "codex-cli", &[model()], false);
    assert!(
        host.session()
            .vendor_ui_state()
            .pending_permission
            .is_some()
    );
    connecting(&mut host);

    assert!(
        host.session()
            .vendor_ui_state()
            .pending_permission
            .is_none()
    );
    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some("model-a")
    );
    Ok(())
}

#[test]
fn ready_keeps_the_restored_model_instead_of_using_provider_default()
-> Result<(), Box<dyn std::error::Error>> {
    let mut host = host()?;
    host.session_mut().set_vendor_ui_state(
        VendorUiState::for_vendor("codex-cli").with_models(vec!["saved".into()], "saved"),
    );

    ready(&mut host, "codex-cli", &[model()], true);

    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some("saved")
    );
    Ok(())
}

#[test]
fn ordinary_ready_uses_provider_selected_model() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = host()?;
    host.session_mut().set_vendor_ui_state(
        VendorUiState::for_vendor("codex-cli").with_models(vec!["saved".into()], "saved"),
    );

    ready(&mut host, "codex-cli", &[model()], false);

    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some("model-a")
    );
    Ok(())
}

#[test]
fn agent_confirmed_model_is_persisted_and_restored() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(ChatHistoryStore::new_session_id());
    let mut session = ChatSession::new();
    session.set_provider_configured("codex-cli");
    session.set_vendor_ui_state(VendorUiState::for_vendor("codex-cli"));
    let mut host = ChatHostState::new(
        session,
        ChatHistoryStore::new(root.clone()),
        ChatOutputExecutor::new(root.clone()),
    )?;

    ready(&mut host, "codex-cli", &[model()], false);
    host.persist()?;

    let restored = ChatHostState::new(
        ChatSession::new(),
        ChatHistoryStore::new(root.clone()),
        ChatOutputExecutor::new(root),
    )?;

    assert_eq!(
        restored
            .session()
            .vendor_ui_state()
            .selected_model
            .as_deref(),
        Some("model-a")
    );
    Ok(())
}

fn host() -> Result<ChatHostState, Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(ChatHistoryStore::new_session_id());
    let mut session = ChatSession::new();
    session.set_provider_configured("codex-cli");
    session.set_vendor_ui_state(VendorUiState::for_vendor("codex-cli"));
    Ok(ChatHostState::new(
        session,
        ChatHistoryStore::new(root.clone()),
        ChatOutputExecutor::new(root),
    )?)
}

fn active_host_with_permission() -> Result<ChatHostState, Box<dyn std::error::Error>> {
    let mut host = host()?;
    host.session_mut().set_provider_configured("codex-cli");
    host.session_mut().set_vendor_ui_state(
        VendorUiState::for_vendor("codex-cli")
            .with_models(vec!["model-a".into(), "model-b".into()], "model-a"),
    );
    host.session_mut().start_assistant_stream(String::new())?;
    project(&mut host);
    Ok(host)
}

fn project(host: &mut ChatHostState) {
    let mut queue = permission::PendingPermissionQueue::default();
    permission::enqueue(host, &mut queue, 7, context(), options());
}

fn assert_active_state(host: &ChatHostState) {
    assert_eq!(
        host.session().render_model().provider,
        ProviderConnectionState::Configured("codex-cli".into())
    );
    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some("model-a")
    );
    assert!(
        host.session()
            .vendor_ui_state()
            .pending_permission
            .is_some()
    );
    assert!(host.session().render_model().input.can_cancel);
}

fn options() -> Vec<AcpPermissionOption> {
    vec![AcpPermissionOption {
        option_id: "allow".to_string(),
        name: "Allow".to_string(),
    }]
}

fn context() -> AcpPermissionContext {
    AcpPermissionContext {
        tool_call_id: "tool-7".into(),
        title: Some("Run command".into()),
        details: vec!["command: cat ./README.md".into()],
    }
}

fn model() -> AcpProviderModel {
    AcpProviderModel {
        config_id: "model".to_string(),
        id: "model-a".to_string(),
        name: "Model A".to_string(),
        selected: true,
    }
}
