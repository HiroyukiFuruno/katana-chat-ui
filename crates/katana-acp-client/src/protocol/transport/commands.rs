use super::{
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
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::mpsc;

pub(crate) async fn command_loop(
    connection: ConnectionTo<Agent>,
    mut commands: mpsc::Receiver<AcpTransportCommand>,
    pending: PendingPermissions,
    capabilities: AcpPromptCapabilities,
    session: (String, BTreeSet<String>),
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let busy = Arc::new(AtomicBool::new(false));
    while let Some(command) = commands.recv().await {
        if matches!(command, AcpTransportCommand::Close) {
            break;
        }
        if let Err(reason) = dispatch(
            &connection,
            command,
            &pending,
            capabilities,
            &session,
            &busy,
            &events,
        ) {
            let _ = events
                .send(AcpTransportEvent::CommandRejected {
                    operation: "ACP command".to_string(),
                    reason,
                })
                .await;
        }
    }
    cancel_permissions(&pending);
    Ok(())
}

#[cfg(test)]
mod tests;

fn dispatch(
    connection: &ConnectionTo<Agent>,
    command: AcpTransportCommand,
    pending: &PendingPermissions,
    capabilities: AcpPromptCapabilities,
    session: &(String, BTreeSet<String>),
    busy: &Arc<AtomicBool>,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<(), String> {
    match command {
        AcpTransportCommand::Prompt(blocks) => {
            start_prompt(connection, blocks, capabilities, &session.0, busy, events)
        }
        AcpTransportCommand::Cancel => cancel_turn(connection, pending, &session.0),
        AcpTransportCommand::SetConfig { config_id, value } => {
            set_config(connection, &session.0, &session.1, config_id, value, events)
        }
        AcpTransportCommand::RespondPermission {
            request_id,
            decision,
        } => resolve_permission(pending, request_id, decision),
        AcpTransportCommand::Close => Ok(()),
    }
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
    connection: &ConnectionTo<Agent>,
    session_id: &str,
    ids: &BTreeSet<String>,
    config_id: String,
    value: String,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<(), String> {
    if !ids.contains(&config_id) {
        return Err("session configuration was not advertised".to_string());
    }
    connection
        .spawn(config_task(
            connection.clone(),
            session_id.to_string(),
            config_id,
            value,
            events.clone(),
        ))
        .map_err(|error| error.to_string())
}

async fn config_task(
    connection: ConnectionTo<Agent>,
    session_id: String,
    config_id: String,
    value: String,
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let result = connection
        .send_request(SetSessionConfigOptionRequest::new(
            session_id.clone(),
            config_id.clone(),
            SessionConfigValueId::new(value),
        ))
        .block_task()
        .await;
    for event in config_result_events(result, session_id, config_id) {
        let _ = events.send(event).await;
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
