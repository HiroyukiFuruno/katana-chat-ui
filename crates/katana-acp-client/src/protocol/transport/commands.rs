use super::{
    config_state::SessionConfigState,
    session_config::project_config_options,
    state::{PendingPermissions, cancel_permissions, resolve_permission},
    types::{AcpTransportCommand, AcpTransportEvent},
};
use crate::protocol::{AcpContentEncoder, AcpPromptCapabilities};
use agent_client_protocol::schema::v1::{
    CancelNotification, ContentBlock, PromptRequest, PromptResponse, SessionConfigValueId,
    SetSessionConfigOptionRequest, SetSessionConfigOptionResponse,
};
use agent_client_protocol::{Agent, ConnectionTo};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const CONFIG_QUEUE_CAPACITY: usize = 16;

struct ConfigRequest {
    connection: ConnectionTo<Agent>,
    session_id: String,
    config_id: String,
    value: String,
    configs: SessionConfigState,
    events: mpsc::Sender<AcpTransportEvent>,
}

struct ConfigWorkerGuard(JoinHandle<()>);

impl Drop for ConfigWorkerGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct CommandContext<'a> {
    connection: &'a ConnectionTo<Agent>,
    pending: &'a PendingPermissions,
    capabilities: AcpPromptCapabilities,
    session: &'a (String, SessionConfigState),
    busy: &'a Arc<AtomicBool>,
    events: &'a mpsc::Sender<AcpTransportEvent>,
    config_queue: &'a mpsc::Sender<ConfigRequest>,
}

pub(crate) async fn command_loop(
    connection: ConnectionTo<Agent>,
    mut commands: mpsc::Receiver<AcpTransportCommand>,
    pending: PendingPermissions,
    capabilities: AcpPromptCapabilities,
    session: (String, SessionConfigState),
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let busy = Arc::new(AtomicBool::new(false));
    let (config_queue, config_requests) = mpsc::channel(CONFIG_QUEUE_CAPACITY);
    let _config_worker = ConfigWorkerGuard(tokio::spawn(config_worker(config_requests)));
    while let Some(command) = commands.recv().await {
        if matches!(command, AcpTransportCommand::Close) {
            break;
        }
        let context = CommandContext {
            connection: &connection,
            pending: &pending,
            capabilities,
            session: &session,
            busy: &busy,
            events: &events,
            config_queue: &config_queue,
        };
        if let Err(event) = dispatch(&context, command) {
            let _ = events.send(*event).await;
        }
    }
    cancel_permissions(&pending);
    Ok(())
}

#[cfg(test)]
mod tests;

fn dispatch(
    context: &CommandContext<'_>,
    command: AcpTransportCommand,
) -> Result<(), Box<AcpTransportEvent>> {
    match command {
        AcpTransportCommand::Prompt(blocks) => start_prompt(
            context.connection,
            blocks,
            context.capabilities,
            &context.session.0,
            context.busy,
            context.events,
        )
        .map_err(command_rejected),
        AcpTransportCommand::Cancel => {
            cancel_turn(context.connection, context.pending, &context.session.0)
                .map_err(command_rejected)
        }
        AcpTransportCommand::SetConfig { config_id, value } => set_config(
            context.config_queue,
            context.connection,
            &context.session.0,
            &context.session.1,
            config_id,
            value,
            context.events,
        ),
        AcpTransportCommand::RespondPermission {
            request_id,
            decision,
        } => resolve_permission(context.pending, request_id, decision).map_err(command_rejected),
        AcpTransportCommand::Close => Ok(()),
    }
}

fn command_rejected(reason: String) -> Box<AcpTransportEvent> {
    Box::new(AcpTransportEvent::CommandRejected {
        operation: "ACP command".to_string(),
        reason,
    })
}

fn start_prompt(
    connection: &ConnectionTo<Agent>,
    blocks: Vec<crate::AcpContentBlock>,
    capabilities: AcpPromptCapabilities,
    session_id: &str,
    busy: &Arc<AtomicBool>,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<(), String> {
    let prompt =
        AcpContentEncoder::encode(&blocks, capabilities).map_err(|error| error.to_string())?;
    if busy.swap(true, Ordering::AcqRel) {
        return Err("a prompt is already active".to_string());
    }
    connection
        .spawn(prompt_task(
            connection.clone(),
            session_id.to_string(),
            prompt,
            busy.clone(),
            events.clone(),
        ))
        .map_err(|error| error.to_string())
}

async fn prompt_task(
    connection: ConnectionTo<Agent>,
    session_id: String,
    prompt: Vec<ContentBlock>,
    busy: Arc<AtomicBool>,
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let result = connection
        .send_request(PromptRequest::new(session_id.clone(), prompt))
        .block_task()
        .await;
    busy.store(false, Ordering::Release);
    send_prompt_result(result, session_id, events).await;
    Ok(())
}

async fn send_prompt_result(
    result: Result<PromptResponse, agent_client_protocol::Error>,
    session_id: String,
    events: mpsc::Sender<AcpTransportEvent>,
) {
    let event = match result {
        Ok(response) => AcpTransportEvent::PromptCompleted {
            session_id,
            stop_reason: format!("{:?}", response.stop_reason),
        },
        Err(error) => AcpTransportEvent::TransportError {
            message: error.to_string(),
        },
    };
    let _ = events.send(event).await;
}

fn cancel_turn(
    connection: &ConnectionTo<Agent>,
    pending: &PendingPermissions,
    session_id: &str,
) -> Result<(), String> {
    cancel_permissions(pending);
    connection
        .send_notification(CancelNotification::new(session_id.to_owned()))
        .map_err(|error| error.to_string())
}

fn set_config(
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

async fn config_worker(mut requests: mpsc::Receiver<ConfigRequest>) {
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

fn validate_config_id(
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

async fn publish_config_result(
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

fn config_result_events(
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
