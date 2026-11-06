use super::*;

fn model_option(id: &str) -> SessionConfigOption {
    SessionConfigOption::select(
        id.to_string(),
        "Model",
        format!("{id}-value"),
        vec![SessionConfigSelectOption::new(
            format!("{id}-value"),
            "Model",
        )],
    )
    .category(SessionConfigOptionCategory::Model)
}

const CONFIG_EVENT_BUFFER: usize = 4;

async fn seed_old_config(
    callbacks: &ConnectionCallbacks,
    received: &mut mpsc::Receiver<AcpTransportEvent>,
) -> TestResult {
    callbacks
        .configs
        .publish(
            AcpTransportEvent::SessionConfigOptionsUpdated {
                session_id: "session-1".to_string(),
                config_options: vec![crate::protocol::AcpSessionConfig {
                    id: "old".to_string(),
                    name: "Old".to_string(),
                    category: crate::protocol::AcpSessionConfigCategory::Other,
                    current_value: None,
                    values: Vec::new(),
                }],
            },
            &callbacks.events,
        )
        .await?;
    let _ = received.recv().await.ok_or("missing old config event")?;
    Ok(())
}

fn expected_new_config_event() -> AcpTransportEvent {
    AcpTransportEvent::SessionConfigOptionsUpdated {
        session_id: "session-1".to_string(),
        config_options: vec![crate::protocol::AcpSessionConfig {
            id: "new".to_string(),
            name: "Model".to_string(),
            category: crate::protocol::AcpSessionConfigCategory::Model,
            current_value: Some("new-value".to_string()),
            values: vec![crate::protocol::AcpConfigValue {
                id: "new-value".to_string(),
                name: "Model".to_string(),
            }],
        }],
    }
}

async fn notification_case() -> TestResult {
    let (events, mut received) = mpsc::channel(CONFIG_EVENT_BUFFER);
    let configs = SessionConfigState::default();
    let callbacks = ConnectionCallbacks::new(events, pending(), configs.clone());
    seed_old_config(&callbacks, &mut received).await?;
    let update =
        SessionUpdate::ConfigOptionUpdate(ConfigOptionUpdate::new(vec![model_option("new")]));
    callbacks
        .notification(SessionNotification::new("session-1", update.clone()))
        .await?;
    assert!(configs.contains("session-1", "new")?);
    assert!(!configs.contains("session-1", "old")?);
    assert_eq!(received.recv().await, Some(expected_new_config_event()));
    assert_eq!(
        received.recv().await,
        Some(AcpTransportEvent::SessionUpdate {
            session_id: "session-1".to_string(),
            update: Box::new(update),
        })
    );
    Ok(())
}

#[test]
fn config_option_notification_updates_state_and_emits_the_same_ui_update() -> TestResult {
    run_async(notification_case())
}

async fn closed_notification_case() -> TestResult {
    let (events, received) = mpsc::channel(1);
    drop(received);
    let configs = SessionConfigState::default();
    let callbacks = ConnectionCallbacks::new(events, pending(), configs.clone());
    let update =
        SessionUpdate::ConfigOptionUpdate(ConfigOptionUpdate::new(vec![model_option("new")]));
    assert!(
        callbacks
            .notification(SessionNotification::new("session-1", update))
            .await
            .is_err()
    );
    assert!(!configs.contains("session-1", "new")?);
    Ok(())
}

#[test]
fn closed_event_sender_keeps_notification_state_unchanged() -> TestResult {
    run_async(closed_notification_case())
}
