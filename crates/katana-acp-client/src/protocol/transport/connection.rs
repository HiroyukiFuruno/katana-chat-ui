mod callbacks;

use super::{
    commands,
    session_config::project_config_options,
    state::PendingPermissions,
    types::{AcpStdioTransportConfig, AcpTransportCommand, AcpTransportEvent},
};
use crate::{AcpError, protocol::AcpPromptCapabilities};
use agent_client_protocol::schema::{
    ProtocolVersion,
    v1::{InitializeRequest, NewSessionRequest},
};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client, ConnectionTo};
use callbacks::ConnectionCallbacks;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

pub(crate) async fn run(
    config: AcpStdioTransportConfig,
    commands: mpsc::Receiver<AcpTransportCommand>,
    events: mpsc::Sender<AcpTransportEvent>,
) {
    let result = connect(config, commands, events.clone()).await;
    if let Err(error) = result {
        let _ = events
            .send(AcpTransportEvent::TransportError {
                message: error.to_string(),
            })
            .await;
    }
    let _ = events.send(AcpTransportEvent::Closed).await;
}

async fn connect(
    config: AcpStdioTransportConfig,
    commands: mpsc::Receiver<AcpTransportCommand>,
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), AcpError> {
    let pending = Arc::new(Mutex::new(BTreeMap::new()));
    let callbacks = ConnectionCallbacks::new(events.clone(), pending.clone());
    let notification_callbacks = callbacks.clone();
    let agent = AcpAgent::new(
        AcpAgentConfig::new(config.command)
            .args(config.args)
            .envs(config.env),
    );
    Client
        .builder()
        .on_receive_notification(
            async move |notification, _| notification_callbacks.notification(notification).await,
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |request, responder, _| callbacks.permission(request, responder).await,
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(agent, move |connection: ConnectionTo<Agent>| {
            start_session(connection, config.cwd, commands, pending, events)
        })
        .await
        .map_err(|error| AcpError::Transport(error.to_string()))
}

async fn start_session(
    connection: ConnectionTo<Agent>,
    cwd: PathBuf,
    commands: mpsc::Receiver<AcpTransportCommand>,
    pending: PendingPermissions,
    events: mpsc::Sender<AcpTransportEvent>,
) -> Result<(), agent_client_protocol::Error> {
    let capabilities = initialize(&connection, &events).await?;
    let session = new_session(&connection, cwd, &events).await?;
    commands::command_loop(connection, commands, pending, capabilities, session, events).await
}

async fn initialize(
    connection: &ConnectionTo<Agent>,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<AcpPromptCapabilities, agent_client_protocol::Error> {
    let response = connection
        .send_request(InitializeRequest::new(ProtocolVersion::V1))
        .block_task()
        .await?;
    if response.protocol_version != ProtocolVersion::V1 {
        return Err(agent_client_protocol::Error::internal_error()
            .data("ACP protocol version is unsupported"));
    }
    let capabilities = AcpPromptCapabilities {
        image: response.agent_capabilities.prompt_capabilities.image,
        embedded_context: response
            .agent_capabilities
            .prompt_capabilities
            .embedded_context,
    };
    let _ = events
        .send(AcpTransportEvent::Initialized {
            protocol_version: response.protocol_version.to_string(),
            image: capabilities.image,
            embedded_context: capabilities.embedded_context,
        })
        .await;
    Ok(capabilities)
}

async fn new_session(
    connection: &ConnectionTo<Agent>,
    cwd: PathBuf,
    events: &mpsc::Sender<AcpTransportEvent>,
) -> Result<(String, BTreeSet<String>), agent_client_protocol::Error> {
    let response = connection
        .send_request(NewSessionRequest::new(cwd))
        .block_task()
        .await?;
    let config_options = response
        .config_options
        .map_or_else(Vec::new, project_config_options);
    let config_ids = config_options
        .iter()
        .map(|option| option.id.clone())
        .collect::<BTreeSet<_>>();
    let session_id = response.session_id.to_string();
    let _ = events
        .send(AcpTransportEvent::SessionStarted {
            session_id: session_id.clone(),
            config_ids: config_ids.iter().cloned().collect(),
            config_options,
        })
        .await;
    Ok((session_id, config_ids))
}

pub(crate) fn validate_config(config: &AcpStdioTransportConfig) -> Result<(), AcpError> {
    if config.cwd.is_absolute() {
        Ok(())
    } else {
        Err(AcpError::Transport(
            "ACP session cwd must be absolute".to_string(),
        ))
    }
}
