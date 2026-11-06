use super::{
    config_state::SessionConfigState,
    state::{
        PendingPermissions, cancel_permissions, reset_permission_cancellation, resolve_permission,
    },
    types::{AcpTransportCommand, AcpTransportEvent},
};
use crate::protocol::{AcpContentEncoder, AcpPromptCapabilities};
use agent_client_protocol::schema::v1::{CancelNotification, PromptRequest, PromptResponse};
use agent_client_protocol::{Agent, ConnectionTo, SentRequest};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

mod config;

use config::{ConfigRequest, config_worker, set_config};
#[cfg(test)]
use config::{config_result_events, publish_config_result, validate_config_id};

const CONFIG_QUEUE_CAPACITY: usize = 16;

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

pub(super) async fn command_loop(
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
        AcpTransportCommand::Prompt(blocks) => {
            start_prompt(context, blocks).map_err(command_rejected)
        }
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
    context: &CommandContext<'_>,
    blocks: Vec<crate::AcpContentBlock>,
) -> Result<(), String> {
    let prompt = AcpContentEncoder::encode(&blocks, context.capabilities)
        .map_err(|error| error.to_string())?;
    if context.busy.swap(true, Ordering::AcqRel) {
        return Err("a prompt is already active".to_string());
    }
    if let Err(error) = reset_permission_cancellation(context.pending) {
        context.busy.store(false, Ordering::Release);
        return Err(error);
    }
    /* WHY: 取消通知が追い越さないよう、要求の登録はtaskへ渡す前に済ませる。 */
    let request = context
        .connection
        .send_request(PromptRequest::new(context.session.0.clone(), prompt));
    context
        .connection
        .spawn(prompt_task(
            request,
            context.session.0.clone(),
            context.busy.clone(),
            context.events.clone(),
        ))
        .map_err(|error| error.to_string())
}

async fn prompt_task(
    request: SentRequest<PromptResponse>,
    session_id: String,
    busy: Arc<AtomicBool>,
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let result = request.block_task().await;
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
