use super::command_rejected;
use crate::protocol::transport::{
    config_state::SessionConfigState, session_config::project_config_options,
    types::AcpTransportEvent,
};
use agent_client_protocol::schema::v1::{
    SessionConfigValueId, SetSessionConfigOptionRequest, SetSessionConfigOptionResponse,
};
use agent_client_protocol::{Agent, ConnectionTo};
use tokio::sync::mpsc;

pub(super) struct ConfigRequest {
    connection: ConnectionTo<Agent>,
    session_id: String,
    config_id: String,
    value: String,
    configs: SessionConfigState,
    events: mpsc::Sender<AcpTransportEvent>,
}

pub(super) fn set_config(
    config_queue: &mpsc::Sender<ConfigRequest>,
    connection: &ConnectionTo<Agent>,
    session_id: &str,
    configs: &SessionConfigState,
    config_id: String,
    value: String,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<(), Box<AcpTransportEvent>> {
    let request = ConfigRequest {
        connection: connection.clone(),
        session_id: session_id.to_string(),
        config_id,
        value,
        configs: configs.clone(),
        events: events.clone(),
    };
    match config_queue.try_send(request) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Full(request)) => {
            Err(Box::new(AcpTransportEvent::ConfigRejected {
                config_id: request.config_id,
                message: "configuration request queue is full; retry later".to_string(),
            }))
        }
        Err(mpsc::error::TrySendError::Closed(_)) => Err(command_rejected(
            "configuration worker is closed".to_string(),
        )),
    }
}

pub(super) async fn config_worker(mut requests: mpsc::Receiver<ConfigRequest>) {
    while let Some(request) = requests.recv().await {
        if process_config_request(request).await.is_err() {
            break;
        }
    }
}

async fn process_config_request(request: ConfigRequest) -> Result<(), ()> {
    if let Err(event) =
        validate_config_id(&request.configs, &request.session_id, &request.config_id)
    {
        request.events.send(*event).await.map_err(|_| ())?;
        return Ok(());
    }
    let result = request
        .connection
        .send_request(SetSessionConfigOptionRequest::new(
            request.session_id.clone(),
            request.config_id.clone(),
            SessionConfigValueId::new(request.value),
        ))
        .block_task()
        .await;
    publish_config_result(
        result,
        request.session_id,
        request.config_id,
        &request.configs,
        &request.events,
    )
    .await
    .map_err(|_| ())
}

pub(super) fn validate_config_id(
    configs: &SessionConfigState,
    session_id: &str,
    config_id: &str,
) -> Result<(), Box<AcpTransportEvent>> {
    if configs
        .contains(session_id, config_id)
        .map_err(command_rejected)?
    {
        Ok(())
    } else {
        Err(Box::new(AcpTransportEvent::ConfigRejected {
            config_id: config_id.to_string(),
            message: "session configuration was not advertised".to_string(),
        }))
    }
}

pub(super) async fn publish_config_result(
    result: Result<SetSessionConfigOptionResponse, agent_client_protocol::Error>,
    session_id: String,
    config_id: String,
    configs: &SessionConfigState,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    for event in config_result_events(result, session_id, config_id) {
        if matches!(event, AcpTransportEvent::SessionConfigOptionsUpdated { .. }) {
            configs.publish(event, events).await?;
        } else {
            events.send(event).await.map_err(|error| {
                agent_client_protocol::Error::internal_error().data(error.to_string())
            })?;
        }
    }
    Ok(())
}

pub(super) fn config_result_events(
    result: Result<SetSessionConfigOptionResponse, agent_client_protocol::Error>,
    session_id: String,
    config_id: String,
) -> Vec<AcpTransportEvent> {
    match result {
        Ok(response) => vec![
            AcpTransportEvent::SessionConfigOptionsUpdated {
                session_id,
                config_options: project_config_options(response.config_options),
            },
            AcpTransportEvent::ConfigUpdated { config_id },
        ],
        Err(error) if is_config_rejection(&error) => {
            vec![AcpTransportEvent::ConfigRejected {
                config_id,
                message: error.message,
            }]
        }
        Err(error) => vec![AcpTransportEvent::TransportError {
            message: error.to_string(),
        }],
    }
}

fn is_config_rejection(error: &agent_client_protocol::Error) -> bool {
    matches!(
        error.code,
        agent_client_protocol::ErrorCode::InvalidRequest
            | agent_client_protocol::ErrorCode::MethodNotFound
            | agent_client_protocol::ErrorCode::InvalidParams
    )
}
