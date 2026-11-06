use crate::{ChatOutputKind, CommandLaunchEntry, HostActionIntent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiOutputHandoffSurface {
    pub label: String,
    pub outputs: Vec<ChatUiOutputSurface>,
    pub json_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiOutputSurface {
    pub id: u64,
    pub source_message_id: u64,
    pub kind: ChatOutputKind,
    #[serde(default)]
    pub status: crate::OutputStatus,
    pub actions: Vec<HostActionIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiSlashLauncherSurface {
    pub visible: bool,
    pub query: String,
    pub entries: Vec<CommandLaunchEntry>,
}
