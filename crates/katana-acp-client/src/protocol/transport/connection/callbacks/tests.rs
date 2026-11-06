#[path = "notification_tests.rs"]
mod notification;

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
