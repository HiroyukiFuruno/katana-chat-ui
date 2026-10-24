use super::super::{ProviderManager, ProviderPollEvent, RestoredModelSelection};
use crate::{
    AcpProviderModel, AcpProviderProjection, ChatHistoryStore, ChatHostState, ChatOutputExecutor,
};
use katana_chat_ui::{ChatSession, ProviderConnectionState, VendorUiState};

#[test]
fn queued_restore_keeps_provider_missing_until_agent_confirms_requested_model()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut manager, mut host, _temp) = awaiting_restore("replacement")?;
    manager
        .inventory
        .ready("codex-cli", vec![model("replacement", true)]);

    assert!(manager.restored_model_is_pending());
    assert_missing_with_saved_model(&host, "saved");

    manager.handle_projection(
        &mut host,
        "codex-cli",
        AcpProviderProjection::ConfigUpdated {
            config_id: "model".into(),
        },
        &mut Vec::<ProviderPollEvent>::new(),
    )?;

    assert!(!manager.restored_model_is_pending());
    assert_eq!(
        host.session().render_model().provider,
        ProviderConnectionState::Configured("codex-cli".into())
    );
    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some("replacement")
    );
    Ok(())
}

#[test]
fn rejected_restored_model_stays_missing_and_keeps_saved_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut manager, mut host, _temp) = awaiting_restore("saved")?;
    let error = manager.handle_projection(
        &mut host,
        "codex-cli",
        AcpProviderProjection::ConfigRejected {
            config_id: "model".into(),
            message: "Invalid params".into(),
        },
        &mut Vec::<ProviderPollEvent>::new(),
    );

    assert!(error.is_err());
    assert_missing_with_saved_model(&host, "saved");
    assert!(matches!(
        manager.restored_model,
        Some(RestoredModelSelection::Blocked { .. })
    ));
    let render_model = host.session().render_model();
    let reason = match &render_model.provider {
        ProviderConnectionState::Missing(reason) => reason,
        ProviderConnectionState::Configured(_) => return Err("provider became ready".into()),
    };
    assert!(reason.contains("Invalid params"));
    Ok(())
}

#[test]
fn restored_model_confirmation_mismatch_stays_missing_and_keeps_saved_selection()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut manager, mut host, _temp) = awaiting_restore("replacement")?;
    manager
        .inventory
        .ready("codex-cli", vec![model("provider-default", true)]);

    let error = manager.handle_projection(
        &mut host,
        "codex-cli",
        AcpProviderProjection::ConfigUpdated {
            config_id: "model".into(),
        },
        &mut Vec::<ProviderPollEvent>::new(),
    );

    assert!(error.is_err());
    assert_missing_with_saved_model(&host, "saved");
    let render_model = host.session().render_model();
    let reason = match &render_model.provider {
        ProviderConnectionState::Missing(reason) => reason,
        ProviderConnectionState::Configured(_) => return Err("provider became ready".into()),
    };
    assert!(reason.contains("replacement"));
    assert!(matches!(
        manager.restored_model,
        Some(RestoredModelSelection::Blocked { .. })
    ));
    Ok(())
}

fn awaiting_restore(
    requested_model: &str,
) -> Result<(ProviderManager, ChatHostState, tempfile::TempDir), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let mut session = ChatSession::new();
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("codex-cli")
            .with_models(vec!["saved".into(), requested_model.into()], "saved"),
    );
    session.set_provider_missing("restored model is being applied");
    let host = ChatHostState::new(
        session,
        ChatHistoryStore::new(root.path().into()),
        ChatOutputExecutor::new(root.path().into()),
    )?;

    let mut manager = ProviderManager::new();
    manager.active_vendor_id = Some("codex-cli".into());
    manager.restored_model = Some(RestoredModelSelection::Applying {
        vendor_id: "codex-cli".into(),
        model_id: "saved".into(),
        config_id: "model".into(),
    });
    manager.await_restored_model_confirmation("codex-cli", "model".into(), requested_model.into());
    Ok((manager, host, root))
}

fn assert_missing_with_saved_model(host: &ChatHostState, expected_model: &str) {
    assert!(!host.session().render_model().provider.is_configured());
    assert_eq!(
        host.session().vendor_ui_state().selected_model.as_deref(),
        Some(expected_model)
    );
}

fn model(id: &str, selected: bool) -> AcpProviderModel {
    AcpProviderModel {
        config_id: "model".into(),
        id: id.into(),
        name: id.into(),
        selected,
    }
}
