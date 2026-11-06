#[path = "prompt_tests.rs"]
mod prompt;
#[path = "wire_tests.rs"]
mod wire;
#[path = "worker_tests.rs"]
mod worker;

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
use agent_client_protocol::{
    Agent, Channel, Client, ConnectionTo, RawJsonRpcMessage, Responder, TransportFrame,
};
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

const CONFIG_EVENT_BUFFER: usize = 4;

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
    let (events, mut received) = tokio::sync::mpsc::channel(CONFIG_EVENT_BUFFER);
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
