use super::*;

struct DropProbe(Option<tokio::sync::oneshot::Sender<()>>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

async fn worker_guard_drop_case() -> TestResult {
    let (started, started_rx) = tokio::sync::oneshot::channel();
    let (dropped, dropped_rx) = tokio::sync::oneshot::channel();
    let worker = tokio::spawn(async move {
        let _probe = DropProbe(Some(dropped));
        let _ = started.send(());
        pending::<()>().await;
    });
    started_rx.await?;
    let guard = super::super::ConfigWorkerGuard(worker);
    drop(guard);
    tokio::time::timeout(Duration::from_secs(1), dropped_rx).await??;
    Ok(())
}

#[test]
fn config_worker_guard_aborts_pending_worker_on_drop() -> TestResult {
    run_async(worker_guard_drop_case())
}

async fn replace_config_and_validate(
    configs: &super::super::super::config_state::SessionConfigState,
    events: &tokio::sync::mpsc::Sender<AcpTransportEvent>,
) -> TestResult {
    configs
        .publish(
            AcpTransportEvent::SessionConfigOptionsUpdated {
                session_id: "session-1".into(),
                config_options: vec![AcpSessionConfig {
                    id: "replacement".into(),
                    name: "Replacement".into(),
                    category: AcpSessionConfigCategory::Other,
                    current_value: None,
                    values: Vec::new(),
                }],
            },
            events,
        )
        .await?;
    assert_rejected(
        super::super::validate_config_id(configs, "session-1", "model"),
        "model",
    );
    assert!(super::super::validate_config_id(configs, "session-1", "replacement").is_ok());
    assert_rejected(
        super::super::validate_config_id(configs, "foreign-session", "replacement"),
        "replacement",
    );
    Ok(())
}

async fn config_id_validation_case() -> TestResult {
    let configs = seeded(vec![confirmed_model_config()]).await?;
    let (events, mut received) = tokio::sync::mpsc::channel(2);
    assert!(super::super::validate_config_id(&configs, "session-1", "model").is_ok());
    assert_rejected(
        super::super::validate_config_id(&configs, "session-1", "unknown"),
        "unknown",
    );
    replace_config_and_validate(&configs, &events).await?;
    assert!(matches!(
        received.try_recv(),
        Ok(AcpTransportEvent::SessionConfigOptionsUpdated { session_id, config_options })
            if session_id == "session-1" && config_options[0].id == "replacement"
    ));
    Ok(())
}

#[test]
fn config_id_validation_accepts_new_and_rejects_removed_or_unknown_ids() -> TestResult {
    run_async(config_id_validation_case())
}
