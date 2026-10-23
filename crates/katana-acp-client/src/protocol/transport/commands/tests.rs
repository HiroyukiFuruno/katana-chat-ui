type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

use std::future::{Future, pending};
use std::time::Duration;

fn run_async(future: impl Future<Output = TestResult>) -> TestResult {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)
}

use agent_client_protocol::schema::v1::{
    CancelNotification, SessionConfigOption, SessionConfigOptionCategory,
    SessionConfigSelectOption, SetSessionConfigOptionRequest, SetSessionConfigOptionResponse,
};
use agent_client_protocol::{Agent, Channel, Client, ConnectionTo, Responder};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::config_result_events;
use crate::protocol::{
    AcpConfigValue, AcpPromptCapabilities, AcpSessionConfig, AcpSessionConfigCategory,
    AcpTransportEvent,
};

#[test]
fn successful_model_change_projects_agent_confirmed_config_and_acknowledges() {
    let events = config_result_events(
        Ok(confirmed_model_response()),
        "session-1".into(),
        "model".into(),
    );

    assert_eq!(events, expected_confirmed_model_events());
}

fn confirmed_model_response() -> SetSessionConfigOptionResponse {
    confirmed_model_response_for("model-b")
}

fn confirmed_model_response_for(
    current_value: impl Into<String>,
) -> SetSessionConfigOptionResponse {
    let model = SessionConfigOption::select(
        "model",
        "Model",
        current_value.into(),
        vec![
            SessionConfigSelectOption::new("model-a", "Model A"),
            SessionConfigSelectOption::new("model-b", "Model B"),
        ],
    )
    .category(SessionConfigOptionCategory::Model);
    SetSessionConfigOptionResponse::new(vec![model])
}

fn expected_confirmed_model_events() -> Vec<AcpTransportEvent> {
    vec![
        AcpTransportEvent::SessionConfigOptionsUpdated {
            session_id: "session-1".into(),
            config_options: vec![confirmed_model_config()],
        },
        AcpTransportEvent::ConfigUpdated {
            config_id: "model".into(),
        },
    ]
}

fn confirmed_model_config() -> AcpSessionConfig {
    AcpSessionConfig {
        id: "model".into(),
        name: "Model".into(),
        category: AcpSessionConfigCategory::Model,
        current_value: Some("model-b".into()),
        values: vec![
            AcpConfigValue {
                id: "model-a".into(),
                name: "Model A".into(),
            },
            AcpConfigValue {
                id: "model-b".into(),
                name: "Model B".into(),
            },
        ],
    }
}

fn legacy_model_config() -> AcpSessionConfig {
    let mut config = confirmed_model_config();
    config.id = "legacy-model".into();
    config
}

#[test]
fn failed_model_change_keeps_config_state_unchanged() {
    let events = config_result_events(
        Err(agent_client_protocol::Error::internal_error()),
        "session-1".into(),
        "model".into(),
    );

    assert!(matches!(
        events.as_slice(),
        [AcpTransportEvent::TransportError { .. }]
    ));
}

#[test]
fn rejected_model_change_is_distinct_from_transport_failure() {
    for error in [
        agent_client_protocol::Error::invalid_request(),
        agent_client_protocol::Error::method_not_found(),
        agent_client_protocol::Error::invalid_params(),
    ] {
        let events = config_result_events(Err(error), "session-1".into(), "model".into());

        assert!(matches!(
            events.as_slice(),
            [AcpTransportEvent::ConfigRejected {
                config_id,
                message: _
            }] if config_id == "model"
        ));
    }
}

async fn seeded(
    config_options: Vec<AcpSessionConfig>,
) -> Result<super::super::config_state::SessionConfigState, Box<dyn std::error::Error>> {
    let configs = super::super::config_state::SessionConfigState::default();
    let (events, mut received) = tokio::sync::mpsc::channel(2);
    configs
        .publish(
            AcpTransportEvent::SessionConfigOptionsUpdated {
                session_id: "session-1".into(),
                config_options,
            },
            &events,
        )
        .await?;
    let _ = received.recv().await.ok_or("missing seed event")?;
    Ok(configs)
}

async fn assert_success_events(
    received: &mut tokio::sync::mpsc::Receiver<AcpTransportEvent>,
) -> TestResult {
    assert_eq!(
        received.recv().await,
        Some(AcpTransportEvent::SessionConfigOptionsUpdated {
            session_id: "session-1".into(),
            config_options: vec![confirmed_model_config()],
        })
    );
    assert_eq!(
        received.recv().await,
        Some(AcpTransportEvent::ConfigUpdated {
            config_id: "model".into(),
        })
    );
    Ok(())
}

fn assert_rejected(result: Result<(), Box<AcpTransportEvent>>, config_id: &str) {
    assert!(matches!(
        result,
        Err(event)
            if matches!(event.as_ref(), AcpTransportEvent::ConfigRejected { config_id: id, .. }
                if id == config_id)
    ));
}

async fn successful_config_result_case() -> TestResult {
    let configs = seeded(vec![legacy_model_config()]).await?;
    let (events, mut received) = tokio::sync::mpsc::channel(4);
    super::publish_config_result(
        Ok(confirmed_model_response()),
        "session-1".into(),
        "model".into(),
        &configs,
        &events,
    )
    .await?;
    assert!(configs.contains("session-1", "model")?);
    assert!(!configs.contains("session-1", "legacy-model")?);
    assert_success_events(&mut received).await
}

#[test]
fn successful_config_result_accepts_new_ids_and_publishes_ui_update() -> TestResult {
    run_async(successful_config_result_case())
}

async fn failed_config_result_case() -> TestResult {
    let configs = seeded(vec![legacy_model_config()]).await?;
    let (events, mut received) = tokio::sync::mpsc::channel(2);
    super::publish_config_result(
        Err(agent_client_protocol::Error::internal_error()),
        "session-1".into(),
        "model".into(),
        &configs,
        &events,
    )
    .await?;
    assert!(configs.contains("session-1", "legacy-model")?);
    assert!(!configs.contains("session-1", "model")?);
    assert!(matches!(
        received.recv().await,
        Some(AcpTransportEvent::TransportError { .. })
    ));
    Ok(())
}

#[test]
fn failed_config_result_keeps_the_advertised_candidate() -> TestResult {
    run_async(failed_config_result_case())
}

async fn closed_config_sender_case() -> TestResult {
    let configs = seeded(vec![legacy_model_config()]).await?;
    let (events, received) = tokio::sync::mpsc::channel(1);
    drop(received);
    assert!(
        super::publish_config_result(
            Ok(confirmed_model_response()),
            "session-1".into(),
            "model".into(),
            &configs,
            &events,
        )
        .await
        .is_err()
    );
    assert!(configs.contains("session-1", "legacy-model")?);
    assert!(!configs.contains("session-1", "model")?);
    Ok(())
}

#[test]
fn closed_event_sender_keeps_config_state_unchanged() -> TestResult {
    run_async(closed_config_sender_case())
}

fn reject_closed_config_queue(
    connection: &ConnectionTo<Agent>,
) -> Result<(), Box<AcpTransportEvent>> {
    let (queue, queue_rx) = tokio::sync::mpsc::channel(1);
    drop(queue_rx);
    let configs = super::super::config_state::SessionConfigState::default();
    let (events, _received) = tokio::sync::mpsc::channel(1);
    super::set_config(
        &queue,
        connection,
        "session-1",
        &configs,
        "model".into(),
        "model-b".into(),
        &events,
    )
}

async fn closed_config_queue_case() -> TestResult {
    let (client_transport, _peer) = Channel::duplex();
    let result = Client
        .builder()
        .connect_with(client_transport, async move |connection| {
            Ok(reject_closed_config_queue(&connection))
        })
        .await?;
    assert!(matches!(result, Err(event) if matches!(
        &*event,
        AcpTransportEvent::CommandRejected { reason, .. }
            if reason == "configuration worker is closed"
    )));
    Ok(())
}

#[test]
fn closed_config_queue_is_rejected_through_set_config() -> TestResult {
    run_async(closed_config_queue_case())
}

async fn prompt_resets_permission_cancellation_case() -> TestResult {
    let (client_transport, _peer) = Channel::duplex();
    let pending = super::super::state::PendingPermissions::default();
    super::super::state::cancel_permissions(&pending);
    let busy = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let (events, _received) = tokio::sync::mpsc::channel(2);
    let (config_queue, _config_requests) = tokio::sync::mpsc::channel(1);
    let session = (
        "session-1".to_string(),
        super::super::config_state::SessionConfigState::default(),
    );
    Client
        .builder()
        .connect_with(client_transport, async move |connection| {
            let context = prompt_context(
                &connection,
                &pending,
                &session,
                &busy,
                &events,
                &config_queue,
            );
            assert_busy_prompt_keeps_cancellation(&context, &pending);
            assert_accepted_prompt_resets_cancellation(&context, &pending, &busy);
            Ok(())
        })
        .await?;
    Ok(())
}

fn assert_busy_prompt_keeps_cancellation(
    context: &super::CommandContext<'_>,
    pending: &super::super::state::PendingPermissions,
) {
    assert!(super::start_prompt(context, Vec::new()).is_err());
    assert_late_permission_cancelled(pending);
}

fn assert_accepted_prompt_resets_cancellation(
    context: &super::CommandContext<'_>,
    pending: &super::super::state::PendingPermissions,
    busy: &Arc<std::sync::atomic::AtomicBool>,
) {
    busy.store(false, std::sync::atomic::Ordering::Release);
    assert!(super::start_prompt(context, Vec::new()).is_ok());
    assert_permission_registered(pending);
}

fn prompt_context<'a>(
    connection: &'a ConnectionTo<Agent>,
    pending: &'a super::super::state::PendingPermissions,
    session: &'a (String, super::super::config_state::SessionConfigState),
    busy: &'a Arc<std::sync::atomic::AtomicBool>,
    events: &'a tokio::sync::mpsc::Sender<AcpTransportEvent>,
    config_queue: &'a tokio::sync::mpsc::Sender<super::ConfigRequest>,
) -> super::CommandContext<'a> {
    super::CommandContext {
        connection,
        pending,
        capabilities: AcpPromptCapabilities {
            image: false,
            embedded_context: false,
        },
        session,
        busy,
        events,
        config_queue,
    }
}

fn assert_late_permission_cancelled(pending: &super::super::state::PendingPermissions) {
    let (sender, mut receiver) = tokio::sync::oneshot::channel();
    assert!(super::super::state::insert_permission(pending, 1, sender).is_ok());
    assert!(matches!(
        receiver.try_recv(),
        Ok(crate::protocol::transport::AcpPermissionDecision::Cancelled)
    ));
}

fn assert_permission_registered(pending: &super::super::state::PendingPermissions) {
    let (sender, _receiver) = tokio::sync::oneshot::channel();
    assert!(matches!(
        super::super::state::insert_permission(pending, 2, sender),
        Ok(super::super::state::PermissionRegistration::Registered)
    ));
}

#[test]
fn accepted_prompt_resets_permission_cancellation_but_busy_prompt_does_not() -> TestResult {
    run_async(prompt_resets_permission_cancellation_case())
}

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
    let guard = super::ConfigWorkerGuard(worker);
    drop(guard);
    tokio::time::timeout(Duration::from_secs(1), dropped_rx).await??;
    Ok(())
}

#[test]
fn config_worker_guard_aborts_pending_worker_on_drop() -> TestResult {
    run_async(worker_guard_drop_case())
}

async fn replace_config_and_validate(
    configs: &super::super::config_state::SessionConfigState,
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
        super::validate_config_id(configs, "session-1", "model"),
        "model",
    );
    assert!(super::validate_config_id(configs, "session-1", "replacement").is_ok());
    assert_rejected(
        super::validate_config_id(configs, "foreign-session", "replacement"),
        "replacement",
    );
    Ok(())
}

async fn config_id_validation_case() -> TestResult {
    let configs = seeded(vec![confirmed_model_config()]).await?;
    let (events, mut received) = tokio::sync::mpsc::channel(2);
    assert!(super::validate_config_id(&configs, "session-1", "model").is_ok());
    assert_rejected(
        super::validate_config_id(&configs, "session-1", "unknown"),
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
    configs: super::super::config_state::SessionConfigState,
    events: tokio::sync::mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let pending = super::super::state::PendingPermissions::default();
    let capabilities = crate::protocol::AcpPromptCapabilities {
        image: false,
        embedded_context: false,
    };
    Client
        .builder()
        .connect_with(transport, async move |connection: ConnectionTo<Agent>| {
            super::command_loop(
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
    for index in 0..17 {
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

async fn seed_wire_configs() -> TestResult<super::super::config_state::SessionConfigState> {
    let configs = super::super::config_state::SessionConfigState::default();
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
    configs: super::super::config_state::SessionConfigState,
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

struct WireSetup {
    inputs: WireTaskInputs,
    commands: tokio::sync::mpsc::Sender<crate::protocol::transport::types::AcpTransportCommand>,
    received: tokio::sync::mpsc::Receiver<AcpTransportEvent>,
    first_rx: tokio::sync::mpsc::Receiver<()>,
    second_rx: tokio::sync::mpsc::Receiver<()>,
    cancel_rx: tokio::sync::mpsc::Receiver<()>,
    release: Arc<tokio::sync::Notify>,
    done: tokio::sync::oneshot::Sender<()>,
    seen: Arc<AtomicUsize>,
}

fn prepare_wire_setup(configs: super::super::config_state::SessionConfigState) -> WireSetup {
    let (client_transport, server_transport) = Channel::duplex();
    let (commands, command_rx) = tokio::sync::mpsc::channel(4);
    let (events, received) = tokio::sync::mpsc::channel(64);
    let (first, first_rx) = tokio::sync::mpsc::channel(1);
    let (second, second_rx) = tokio::sync::mpsc::channel(1);
    let seen = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(tokio::sync::Notify::new());
    let state = wire_server_state(&seen, first, second, &release);
    let (cancel, cancel_rx) = tokio::sync::mpsc::channel(1);
    let (done, done_rx) = tokio::sync::oneshot::channel();
    WireSetup {
        inputs: WireTaskInputs {
            client_transport,
            server_transport,
            command_rx,
            configs,
            events,
            state,
            cancel,
            done: done_rx,
        },
        commands,
        received,
        first_rx,
        second_rx,
        cancel_rx,
        release,
        done,
        seen,
    }
}

fn wire_server_state(
    seen: &Arc<AtomicUsize>,
    first: tokio::sync::mpsc::Sender<()>,
    second: tokio::sync::mpsc::Sender<()>,
    release: &Arc<tokio::sync::Notify>,
) -> WireServerState {
    WireServerState {
        seen: Arc::clone(seen),
        first_received: Arc::new(first),
        second_received: Arc::new(second),
        release_first: Arc::clone(release),
    }
}

fn spawn_wire_tasks(inputs: WireTaskInputs) -> WireTaskHandles {
    let WireTaskInputs {
        client_transport,
        server_transport,
        command_rx,
        configs,
        events,
        state,
        cancel,
        done,
    } = inputs;
    WireTaskHandles {
        client: tokio::spawn(run_wire_client(
            client_transport,
            command_rx,
            configs,
            events,
        )),
        server: tokio::spawn(run_wire_server(server_transport, state, cancel, done)),
    }
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
    for index in 0..15 {
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
