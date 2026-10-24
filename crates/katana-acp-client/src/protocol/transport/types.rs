use crate::AcpContentBlock;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpStdioTransportConfig {
    pub command: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cwd: PathBuf,
}

impl AcpStdioTransportConfig {
    pub fn new(command: impl Into<PathBuf>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: cwd.into(),
        }
    }

    #[must_use]
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// ACP agent の子プロセスだけに渡す環境変数を設定する。
    #[must_use]
    pub fn envs(
        mut self,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.env.extend(
            env.into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpPermissionDecision {
    Selected { option_id: String },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpPermissionOption {
    pub option_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpPermissionContext {
    pub tool_call_id: String,
    pub title: Option<String>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpConfigValue {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpSessionConfig {
    pub id: String,
    pub name: String,
    pub category: AcpSessionConfigCategory,
    pub current_value: Option<String>,
    pub values: Vec<AcpConfigValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpSessionConfigCategory {
    Model,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AcpTransportEvent {
    Initialized {
        protocol_version: String,
        image: bool,
        embedded_context: bool,
    },
    SessionStarted {
        session_id: String,
        config_ids: Vec<String>,
        config_options: Vec<AcpSessionConfig>,
    },
    SessionUpdate {
        session_id: String,
        update: Box<agent_client_protocol::schema::v1::SessionUpdate>,
    },
    SessionConfigOptionsUpdated {
        session_id: String,
        config_options: Vec<AcpSessionConfig>,
    },
    PromptCompleted {
        session_id: String,
        stop_reason: String,
    },
    ConfigUpdated {
        config_id: String,
    },
    ConfigRejected {
        config_id: String,
        message: String,
    },
    PermissionRequested {
        request_id: u64,
        session_id: String,
        context: AcpPermissionContext,
        options: Vec<AcpPermissionOption>,
    },
    CommandRejected {
        operation: String,
        reason: String,
    },
    TransportError {
        message: String,
    },
    Closed,
}

#[derive(Debug)]
pub(crate) enum AcpTransportCommand {
    Prompt(Vec<AcpContentBlock>),
    Cancel,
    SetConfig {
        config_id: String,
        value: String,
    },
    RespondPermission {
        request_id: u64,
        decision: AcpPermissionDecision,
    },
    Close,
}
