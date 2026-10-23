use std::path::Path;

use katana_acp_client::protocol::{AcpPermissionContext, AcpPermissionOption};
use katana_chat_ui::{ChatSession, ChatUiAction, VendorUiState};

use super::{AcpHostBridge, AcpProviderProjection};
use crate::{
    AcpProviderExecutor, AcpProviderProfile, ChatHistoryStore, ChatHostState, ChatOutputExecutor,
};

#[test]
fn cancelling_suppresses_residual_stream_and_permission_updates() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let (runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    bridge.streaming = true;
    bridge.cancelling = true;
    let before = host.session().clone();

    for projection in residual_projections() {
        bridge
            .apply_projection(&mut host, projection)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(host.session(), &before);
    assert!(bridge.cancelling);
    assert!(host.session().render_model().input.can_cancel);
    runtime
        .block_on(bridge.stop(&mut host))
        .map_err(|error| error.to_string())?;
    assert!(bridge.cancelling);
    Ok(())
}

#[test]
fn cancelling_projects_confirmed_model_without_finishing_turn() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let (_runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    bridge.streaming = true;
    bridge.cancelling = true;
    host.session_mut().set_vendor_ui_state(
        VendorUiState::for_vendor("codex-cli").with_models(vec!["old".into()], "old"),
    );
    let before = host.session().snapshot().messages;
    let models = vec![crate::AcpProviderModel {
        config_id: "model".into(),
        id: "confirmed".into(),
        name: "Confirmed".into(),
        selected: true,
    }];
    bridge
        .apply_projection(&mut host, AcpProviderProjection::Ready { models })
        .map_err(|error| error.to_string())?;
    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some("confirmed")
    );
    assert!(bridge.cancelling && bridge.streaming);
    assert!(host.session().render_model().input.can_cancel);
    assert_eq!(host.session().snapshot().messages, before);
    assert_eq!(host.session().draft().text, "next draft");
    Ok(())
}

#[test]
fn cancelling_reports_configuration_rejection_without_finishing_turn() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let (_runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    bridge.streaming = true;
    bridge.cancelling = true;
    let before = host.session().clone();
    let result = bridge.apply_projection(
        &mut host,
        AcpProviderProjection::ConfigRejected {
            config_id: "model".into(),
            message: "denied config".into(),
        },
    );
    assert!(
        matches!(result, Err(crate::ChatHostError::Provider(reason)) if reason.contains("denied config"))
    );
    assert!(bridge.cancelling && bridge.streaming);
    assert_eq!(host.session(), &before);
    Ok(())
}

#[test]
fn finished_projection_releases_cancellation_and_completes_stream() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let (_runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    bridge.streaming = true;
    bridge.cancelling = true;
    bridge
        .apply_projection(&mut host, AcpProviderProjection::Finished)
        .map_err(|error| error.to_string())?;

    assert!(!bridge.cancelling);
    assert!(!bridge.streaming);
    assert!(!host.session().render_model().input.can_cancel);
    assert!(bridge.ensure_not_cancelling().is_ok());
    assert!(host.dispatch(ChatUiAction::Submit).is_ok());
    assert!(host.session().draft().text.is_empty());
    assert_eq!(
        host.session()
            .snapshot()
            .messages
            .last()
            .map(|message| message.content.as_str()),
        Some("next draft")
    );
    Ok(())
}

#[test]
fn failed_projection_releases_cancellation_and_fails_stream() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let (_runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    bridge.streaming = true;
    bridge.cancelling = true;
    bridge
        .apply_projection(&mut host, AcpProviderProjection::Failed("cancelled".into()))
        .map_err(|error| error.to_string())?;

    assert!(!bridge.cancelling);
    assert!(!bridge.streaming);
    assert!(!host.session().render_model().input.can_cancel);
    Ok(())
}

#[test]
fn direct_submit_is_rejected_without_session_mutation_while_cancelling()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let (runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    bridge.cancelling = true;
    let before = host.session().clone();
    assert!(host.session().render_model().input.can_cancel);
    assert!(host.dispatch(ChatUiAction::Submit).is_err());
    assert_eq!(host.session(), &before);
    let result = runtime.block_on(bridge.submit(&mut host));

    assert!(result.is_err());
    assert_eq!(host.session(), &before);
    Ok(())
}

#[test]
fn idle_stop_is_non_mutating() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let (runtime, mut bridge, mut host) = bridge_and_host(root.path())?;
    host.session_mut().finish_assistant_message()?;
    let before = host.session().clone();
    runtime.block_on(bridge.stop(&mut host))?;

    assert!(!bridge.cancelling);
    assert_eq!(host.session(), &before);
    Ok(())
}

fn bridge_and_host(
    root: &Path,
) -> Result<(tokio::runtime::Runtime, AcpHostBridge, ChatHostState), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let result = runtime.block_on(async {
        let profile = AcpProviderProfile::new("codex-cli", "/bin/cat", root);
        let executor = AcpProviderExecutor::connect(profile).map_err(|error| error.to_string())?;
        let bridge = AcpHostBridge::new(executor);
        let mut session = ChatSession::new();
        session.set_provider_configured("codex-cli");
        let mut host = ChatHostState::new(
            session,
            ChatHistoryStore::new(root.to_path_buf()),
            ChatOutputExecutor::new(root.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
        host.session_mut()
            .start_assistant_stream(String::new())
            .map_err(|error| error.to_string())?;
        host.session_mut().draft_mut().set_text("next draft");
        Ok::<_, String>((bridge, host))
    })?;
    Ok((runtime, result.0, result.1))
}

fn residual_projections() -> [AcpProviderProjection; 4] {
    [
        AcpProviderProjection::Text("late text".into()),
        AcpProviderProjection::Thinking("late thinking".into()),
        AcpProviderProjection::Activity(katana_chat_ui::AgentActivityKind::Generating),
        AcpProviderProjection::Permission {
            request_id: 7,
            context: AcpPermissionContext {
                tool_call_id: "tool-7".into(),
                title: Some("late permission".into()),
                details: vec!["late".into()],
            },
            options: vec![AcpPermissionOption {
                option_id: "allow".into(),
                name: "Allow".into(),
            }],
        },
    ]
}
