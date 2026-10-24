use super::ConnectionCallbacks;
use crate::protocol::transport::{
    config_state::SessionConfigState,
    state::{PendingPermissions, reset_permission_cancellation, resolve_permission},
    types::{AcpPermissionDecision, AcpTransportEvent},
};
use agent_client_protocol::schema::v1::{
    ConfigOptionUpdate, PermissionOption, PermissionOptionKind, RequestPermissionOutcome,
    RequestPermissionRequest, SessionConfigOption, SessionConfigOptionCategory,
    SessionConfigSelectOption, SessionNotification, SessionUpdate, ToolCallUpdate,
    ToolCallUpdateFields,
};
use agent_client_protocol::{Agent, Channel, Client, ConnectionTo};
use tokio::sync::{mpsc, oneshot};

type TestResult = Result<(), Box<dyn std::error::Error>>;

use std::future::Future;

fn run_async(future: impl Future<Output = TestResult>) -> TestResult {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)
}

fn pending() -> PendingPermissions {
    PendingPermissions::default()
}

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
    let (events, mut received) = mpsc::channel(4);
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

fn permission_request() -> RequestPermissionRequest {
    RequestPermissionRequest::new(
        "session-1",
        ToolCallUpdate::new("tool-1", ToolCallUpdateFields::new().title("late")),
        vec![PermissionOption::new(
            "allow-once",
            "Allow once",
            PermissionOptionKind::AllowOnce,
        )],
    )
}

async fn request_permission(
    connection: &ConnectionTo<Client>,
) -> Result<RequestPermissionOutcome, agent_client_protocol::Error> {
    Ok(connection
        .send_request(permission_request())
        .block_task()
        .await?
        .outcome)
}

async fn run_permission_agent(
    transport: Channel,
    first_done: mpsc::Sender<()>,
    continue_second: oneshot::Receiver<()>,
) -> Result<(), agent_client_protocol::Error> {
    Agent
        .builder()
        .connect_with(transport, async move |connection: ConnectionTo<Client>| {
            assert!(matches!(
                request_permission(&connection).await?,
                RequestPermissionOutcome::Cancelled
            ));
            first_done
                .send(())
                .await
                .map_err(|_| agent_client_protocol::Error::internal_error())?;
            continue_second
                .await
                .map_err(|_| agent_client_protocol::Error::internal_error())?;
            assert!(matches!(
                request_permission(&connection).await?,
                RequestPermissionOutcome::Selected(outcome)
                    if outcome.option_id.to_string() == "allow-once"
            ));
            Ok(())
        })
        .await?;
    Ok(())
}

async fn connect_permission_client(
    transport: Channel,
    callbacks: ConnectionCallbacks,
    finish_receiver: oneshot::Receiver<()>,
) -> Result<(), agent_client_protocol::Error> {
    Client
        .builder()
        .on_receive_request(
            async move |request, responder, _| callbacks.permission(request, responder).await,
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(transport, async move |_| {
            finish_receiver
                .await
                .map_err(|_| agent_client_protocol::Error::internal_error())
        })
        .await
}

async fn resolve_second_permission(
    pending: &PendingPermissions,
    continue_second: oneshot::Sender<()>,
    received: &mut mpsc::Receiver<AcpTransportEvent>,
) -> TestResult {
    reset_permission_cancellation(pending)?;
    continue_second
        .send(())
        .map_err(|_| std::io::Error::other("agent stopped"))?;
    let event = received
        .recv()
        .await
        .ok_or_else(|| std::io::Error::other("missing second UI event"))?;
    assert!(matches!(
        event,
        AcpTransportEvent::PermissionRequested { request_id: 2, .. }
    ));
    resolve_permission(
        pending,
        2,
        AcpPermissionDecision::Selected {
            option_id: "allow-once".to_string(),
        },
    )?;
    Ok(())
}

async fn finish_permission_exchange(
    pending: &PendingPermissions,
    continue_second: oneshot::Sender<()>,
    received: &mut mpsc::Receiver<AcpTransportEvent>,
    agent: tokio::task::JoinHandle<Result<(), agent_client_protocol::Error>>,
    finish_sender: oneshot::Sender<()>,
    client: tokio::task::JoinHandle<Result<(), agent_client_protocol::Error>>,
) -> TestResult {
    resolve_second_permission(pending, continue_second, received).await?;
    agent.await??;
    finish_sender
        .send(())
        .map_err(|_| std::io::Error::other("client stopped"))?;
    client.await??;
    Ok(())
}

async fn assert_first_permission_cancelled(
    first_received: &mut mpsc::Receiver<()>,
    received: &mut mpsc::Receiver<AcpTransportEvent>,
) -> TestResult {
    first_received
        .recv()
        .await
        .ok_or_else(|| std::io::Error::other("missing first response"))?;
    assert!(matches!(
        received.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    ));
    Ok(())
}

fn cancelled_permission_callbacks() -> (
    PendingPermissions,
    ConnectionCallbacks,
    mpsc::Receiver<AcpTransportEvent>,
) {
    let pending = pending();
    crate::protocol::transport::state::cancel_permissions(&pending);
    let (events, received) = mpsc::channel(1);
    let callbacks =
        ConnectionCallbacks::new(events, pending.clone(), SessionConfigState::default());
    (pending, callbacks, received)
}

async fn late_permission_wire_case() -> TestResult {
    let (client_transport, agent_transport) = Channel::duplex();
    let (pending, callbacks, mut received) = cancelled_permission_callbacks();
    let (first_done, mut first_received) = mpsc::channel(1);
    let (continue_second, continue_receiver) = oneshot::channel();
    let agent = tokio::spawn(run_permission_agent(
        agent_transport,
        first_done,
        continue_receiver,
    ));
    let (finish_sender, finish_receiver) = oneshot::channel();
    let client = tokio::spawn(connect_permission_client(
        client_transport,
        callbacks,
        finish_receiver,
    ));
    assert_first_permission_cancelled(&mut first_received, &mut received).await?;
    finish_permission_exchange(
        &pending,
        continue_second,
        &mut received,
        agent,
        finish_sender,
        client,
    )
    .await
}

#[test]
fn late_permission_over_wire_is_cancelled_without_ui_event() -> TestResult {
    run_async(late_permission_wire_case())
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
