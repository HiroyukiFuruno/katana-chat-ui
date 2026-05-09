use serde::{Deserialize, Serialize};

use super::OutputId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputStatus {
    Ready,
    PendingHost,
    Applied,
    Reverted,
    Rejected,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostActionIntent {
    pub output_id: OutputId,
    pub kind: HostActionKind,
}

impl HostActionIntent {
    pub fn new(output_id: OutputId, kind: HostActionKind) -> Self {
        Self { output_id, kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostActionKind {
    Copy,
    OpenPreview,
    CreateFile,
    ApplyDiff,
    UndoChange,
    Approve,
    Reject,
}
