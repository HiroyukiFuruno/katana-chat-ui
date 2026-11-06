use super::*;

#[path = "wire_setup_tests.rs"]
mod setup;
use setup::{prepare_wire_setup, spawn_wire_tasks};

#[derive(Clone)]
struct WireServerState {
    seen: Arc<AtomicUsize>,
    first_received: Arc<tokio::sync::mpsc::Sender<()>>,
    second_received: Arc<tokio::sync::mpsc::Sender<()>>,
    release_first: Arc<tokio::sync::Notify>,
}

async fn handle_config_request(
    request: SetSessionConfigOptionRequest,
    responder: Responder<SetSessionConfigOptionResponse>,
    connection: ConnectionTo<Client>,
    state: WireServerState,
) -> Result<(), agent_client_protocol::Error> {
    let index = state.seen.fetch_add(1, Ordering::SeqCst);
    let value = match request.value {
        agent_client_protocol::schema::v1::SessionConfigOptionValue::ValueId { value } => {
            value.to_string()
        }
        agent_client_protocol::schema::v1::SessionConfigOptionValue::Boolean { value } => {
            value.to_string()
        }
        _ => "unknown".to_string(),
    };
    let first_received = Arc::clone(&state.first_received);
    let second_received = Arc::clone(&state.second_received);
    let release_first = Arc::clone(&state.release_first);
    connection.spawn(async move {
        if index == 0 {
            let _ = first_received.try_send(());
            release_first.notified().await;
        } else {
            let _ = second_received.try_send(());
        }
        responder.respond(confirmed_model_response_for(value))
    })
}

async fn run_wire_server(
    transport: Channel,
    state: WireServerState,
    cancel_received: tokio::sync::mpsc::Sender<()>,
    done: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), agent_client_protocol::Error> {
    Agent
        .builder()
        .on_receive_notification(
            async move |_notification: CancelNotification, _connection| {
                let _ = cancel_received.try_send(());
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |request: SetSessionConfigOptionRequest,
                        responder: Responder<SetSessionConfigOptionResponse>,
                        connection| {
                handle_config_request(request, responder, connection, state.clone()).await
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(transport, async move |_| {
            let _ = done.await;
            Ok(())
        })
        .await
}

async fn run_wire_client(
    transport: Channel,
    command_rx: tokio::sync::mpsc::Receiver<crate::protocol::transport::types::AcpTransportCommand>,
    configs: super::super::super::config_state::SessionConfigState,
    events: tokio::sync::mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let pending = super::super::super::state::PendingPermissions::default();
    let capabilities = crate::protocol::AcpPromptCapabilities {
        image: false,
        embedded_context: false,
    };
    Client
        .builder()
        .connect_with(transport, async move |connection: ConnectionTo<Agent>| {
            super::super::command_loop(
                connection,
                command_rx,
                pending,
                capabilities,
                ("session-1".into(), configs),
                events,
            )
            .await
        })
        .await
}

async fn send_set_config(
    commands: &tokio::sync::mpsc::Sender<crate::protocol::transport::types::AcpTransportCommand>,
    value: String,
) -> TestResult {
    commands
        .send(
            crate::protocol::transport::types::AcpTransportCommand::SetConfig {
                config_id: "model".into(),
                value,
            },
        )
        .await?;
    Ok(())
}

async fn assert_wire_results(
    received: &mut tokio::sync::mpsc::Receiver<AcpTransportEvent>,
) -> TestResult {
    for index in 0..=super::super::CONFIG_QUEUE_CAPACITY {
        assert_wire_result(received, index).await?;
    }
    Ok(())
}

async fn assert_wire_result(
    received: &mut tokio::sync::mpsc::Receiver<AcpTransportEvent>,
    index: usize,
) -> TestResult {
    let expected = match index {
        0 => "model-a".to_string(),
        1 => "model-b".to_string(),
        index => format!("queued-{}", index - 2),
    };
    assert!(matches!(
        received.recv().await,
        Some(AcpTransportEvent::SessionConfigOptionsUpdated { config_options, .. })
            if config_options[0].current_value.as_deref() == Some(expected.as_str())
    ));
    assert!(matches!(
        received.recv().await,
        Some(AcpTransportEvent::ConfigUpdated { config_id }) if config_id == "model"
    ));
    Ok(())
}

async fn seed_wire_configs() -> TestResult<super::super::super::config_state::SessionConfigState> {
    let configs = super::super::super::config_state::SessionConfigState::default();
    let (events, mut received) = tokio::sync::mpsc::channel(2);
    configs
        .publish(
            AcpTransportEvent::SessionConfigOptionsUpdated {
                session_id: "session-1".into(),
                config_options: vec![confirmed_model_config()],
            },
            &events,
        )
        .await?;
    let _ = received.recv().await;
    Ok(configs)
}

struct WireFixture {
    commands: tokio::sync::mpsc::Sender<crate::protocol::transport::types::AcpTransportCommand>,
    received: tokio::sync::mpsc::Receiver<AcpTransportEvent>,
    first_rx: tokio::sync::mpsc::Receiver<()>,
    second_rx: tokio::sync::mpsc::Receiver<()>,
    cancel_rx: tokio::sync::mpsc::Receiver<()>,
    release: Arc<tokio::sync::Notify>,
    done: tokio::sync::oneshot::Sender<()>,
    client: tokio::task::JoinHandle<Result<(), agent_client_protocol::Error>>,
    server: tokio::task::JoinHandle<Result<(), agent_client_protocol::Error>>,
    seen: Arc<AtomicUsize>,
}

struct WireTaskInputs {
    client_transport: Channel,
    server_transport: Channel,
    command_rx: tokio::sync::mpsc::Receiver<crate::protocol::transport::types::AcpTransportCommand>,
    configs: super::super::super::config_state::SessionConfigState,
    events: tokio::sync::mpsc::Sender<AcpTransportEvent>,
    state: WireServerState,
    cancel: tokio::sync::mpsc::Sender<()>,
    done: tokio::sync::oneshot::Receiver<()>,
}

struct WireTaskHandles {
    client: tokio::task::JoinHandle<Result<(), agent_client_protocol::Error>>,
    server: tokio::task::JoinHandle<Result<(), agent_client_protocol::Error>>,
}

async fn start_wire_fixture() -> TestResult<WireFixture> {
    let configs = seed_wire_configs().await?;
    let setup = prepare_wire_setup(configs);
    let WireTaskHandles { client, server } = spawn_wire_tasks(setup.inputs);
    Ok(WireFixture {
        client,
        server,
        commands: setup.commands,
        received: setup.received,
        first_rx: setup.first_rx,
        second_rx: setup.second_rx,
        cancel_rx: setup.cancel_rx,
        release: setup.release,
        done: setup.done,
        seen: setup.seen,
    })
}

async fn enqueue_wire_requests(fixture: &mut WireFixture) -> TestResult {
    send_set_config(&fixture.commands, "model-a".into()).await?;
    fixture
        .first_rx
        .recv()
        .await
        .ok_or("first request missing")?;
    send_set_config(&fixture.commands, "model-b".into()).await?;
    enqueue_queue_values(&fixture.commands).await?;
    assert_queue_full(fixture).await?;
    assert!(fixture.second_rx.try_recv().is_err());
    Ok(())
}

async fn enqueue_queue_values(
    commands: &tokio::sync::mpsc::Sender<crate::protocol::transport::types::AcpTransportCommand>,
) -> TestResult {
    for index in 0..super::super::CONFIG_QUEUE_CAPACITY - 1 {
        send_set_config(commands, format!("queued-{index}")).await?;
    }
    send_set_config(commands, "queue-full".into()).await
}

async fn assert_queue_full(fixture: &mut WireFixture) -> TestResult {
    assert!(matches!(
        fixture.received.recv().await,
        Some(AcpTransportEvent::ConfigRejected { message, .. })
            if message.contains("queue is full")
    ));
    Ok(())
}

async fn finish_wire_fixture(mut fixture: WireFixture) -> TestResult {
    fixture
        .commands
        .send(crate::protocol::transport::types::AcpTransportCommand::Cancel)
        .await?;
    fixture
        .cancel_rx
        .recv()
        .await
        .ok_or("cancel notification missing")?;
    fixture.release.notify_one();
    fixture
        .second_rx
        .recv()
        .await
        .ok_or("second request missing")?;
    assert_wire_results(&mut fixture.received).await?;
    fixture
        .commands
        .send(crate::protocol::transport::types::AcpTransportCommand::Close)
        .await?;
    let _ = fixture.done.send(());
    fixture.client.await??;
    fixture.server.await??;
    assert_eq!(fixture.seen.load(Ordering::SeqCst), 17);
    Ok(())
}

#[test]
fn command_loop_serializes_config_requests_over_real_json_rpc() -> TestResult {
    run_async(async {
        let mut fixture = start_wire_fixture().await?;
        enqueue_wire_requests(&mut fixture).await?;
        finish_wire_fixture(fixture).await?;
        Ok(())
    })
}
