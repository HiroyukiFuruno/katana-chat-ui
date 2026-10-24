use super::{
    connection,
    types::{
        AcpPermissionDecision, AcpStdioTransportConfig, AcpTransportCommand, AcpTransportEvent,
    },
};
use crate::{AcpContentBlock, AcpError};
use tokio::sync::mpsc;

const COMMAND_CAPACITY: usize = 32;
const EVENT_CAPACITY: usize = 64;

pub struct AcpStdioTransport {
    commands: mpsc::Sender<AcpTransportCommand>,
    events: mpsc::Receiver<AcpTransportEvent>,
    task: tokio::task::JoinHandle<()>,
}

impl AcpStdioTransport {
    pub fn spawn(config: AcpStdioTransportConfig) -> Result<Self, AcpError> {
        connection::validate_config(&config)?;
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|error| AcpError::Transport(error.to_string()))?;
        let (commands, command_rx) = mpsc::channel(COMMAND_CAPACITY);
        let (event_tx, events) = mpsc::channel(EVENT_CAPACITY);
        let task = runtime.spawn(connection::run(config, command_rx, event_tx));
        Ok(Self {
            commands,
            events,
            task,
        })
    }

    pub async fn prompt(&self, blocks: Vec<AcpContentBlock>) -> Result<(), AcpError> {
        self.send(AcpTransportCommand::Prompt(blocks)).await
    }

    pub async fn cancel(&self) -> Result<(), AcpError> {
        self.send(AcpTransportCommand::Cancel).await
    }

    pub async fn set_config(&self, config_id: String, value: String) -> Result<(), AcpError> {
        self.send(AcpTransportCommand::SetConfig { config_id, value })
            .await
    }

    pub async fn respond_permission(
        &self,
        request_id: u64,
        decision: AcpPermissionDecision,
    ) -> Result<(), AcpError> {
        self.send(AcpTransportCommand::RespondPermission {
            request_id,
            decision,
        })
        .await
    }

    pub async fn close(&self) -> Result<(), AcpError> {
        self.send(AcpTransportCommand::Close).await
    }

    pub async fn recv(&mut self) -> Option<AcpTransportEvent> {
        self.events.recv().await
    }

    pub fn try_recv(&mut self) -> Option<AcpTransportEvent> {
        self.events.try_recv().ok()
    }

    async fn send(&self, command: AcpTransportCommand) -> Result<(), AcpError> {
        self.commands
            .send(command)
            .await
            .map_err(|_| AcpError::Transport("ACP transport is closed".to_string()))
    }
}

impl Drop for AcpStdioTransport {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::AcpStdioTransportConfig;
    use crate::protocol::transport::connection::validate_config;
    use std::path::PathBuf;

    #[test]
    fn relative_cwd_is_rejected_before_launch() {
        let config = AcpStdioTransportConfig::new("agent", "relative");
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn command_and_arguments_stay_separate() {
        let config = AcpStdioTransportConfig::new("agent", PathBuf::from("/workspace"))
            .args(["serve", "--stdio"])
            .envs([("KCU_PROVIDER_MODE", "local")]);
        assert_eq!(config.command, PathBuf::from("agent"));
        assert_eq!(config.args, ["serve", "--stdio"]);
        assert_eq!(
            config.env.get("KCU_PROVIDER_MODE"),
            Some(&"local".to_string())
        );
    }
}
