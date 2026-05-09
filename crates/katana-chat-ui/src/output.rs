use serde::{Deserialize, Serialize};

mod action;
mod payload;

pub use action::{HostActionIntent, HostActionKind, OutputStatus};
pub use payload::{
    CodeOutput, DiffCandidateOutput, FileCandidateOutput, PermissionRequestOutput, TextOutput,
    ToolResultOutput,
};

pub type OutputId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatOutput {
    pub id: OutputId,
    pub source_message_id: u64,
    pub kind: ChatOutputKind,
    pub status: OutputStatus,
}

impl ChatOutput {
    pub fn new(id: OutputId, source_message_id: u64, kind: ChatOutputKind) -> Self {
        Self {
            id,
            source_message_id,
            kind,
            status: OutputStatus::Ready,
        }
    }

    pub fn host_actions(&self) -> Vec<HostActionIntent> {
        self.kind.host_actions(self.id, &self.status)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatOutputKind {
    Text(TextOutput),
    Code(CodeOutput),
    FileCandidate(FileCandidateOutput),
    DiffCandidate(DiffCandidateOutput),
    ToolResult(ToolResultOutput),
    PermissionRequest(PermissionRequestOutput),
}

impl ChatOutputKind {
    fn host_actions(&self, output_id: OutputId, status: &OutputStatus) -> Vec<HostActionIntent> {
        if let Some(actions) = self.status_actions(output_id, status) {
            return actions;
        }
        self.default_actions(output_id)
    }

    fn status_actions(
        &self,
        output_id: OutputId,
        status: &OutputStatus,
    ) -> Option<Vec<HostActionIntent>> {
        match status {
            OutputStatus::Applied if self.can_undo() => Some(vec![
                HostActionIntent::new(output_id, HostActionKind::OpenPreview),
                HostActionIntent::new(output_id, HostActionKind::UndoChange),
            ]),
            OutputStatus::Failed(_) | OutputStatus::Rejected | OutputStatus::Reverted
                if self.is_mutating_candidate() =>
            {
                Some(vec![HostActionIntent::new(
                    output_id,
                    HostActionKind::OpenPreview,
                )])
            }
            _ => None,
        }
    }

    fn default_actions(&self, output_id: OutputId) -> Vec<HostActionIntent> {
        match self {
            Self::Text(_) | Self::Code(_) | Self::ToolResult(_) => {
                vec![HostActionIntent::new(output_id, HostActionKind::Copy)]
            }
            Self::FileCandidate(_) => vec![
                HostActionIntent::new(output_id, HostActionKind::OpenPreview),
                HostActionIntent::new(output_id, HostActionKind::CreateFile),
            ],
            Self::DiffCandidate(_) => vec![
                HostActionIntent::new(output_id, HostActionKind::OpenPreview),
                HostActionIntent::new(output_id, HostActionKind::ApplyDiff),
            ],
            Self::PermissionRequest(_) => vec![
                HostActionIntent::new(output_id, HostActionKind::Approve),
                HostActionIntent::new(output_id, HostActionKind::Reject),
            ],
        }
    }

    fn can_undo(&self) -> bool {
        matches!(self, Self::FileCandidate(_) | Self::DiffCandidate(_))
    }

    fn is_mutating_candidate(&self) -> bool {
        matches!(
            self,
            Self::FileCandidate(_) | Self::DiffCandidate(_) | Self::PermissionRequest(_)
        )
    }
}

#[cfg(test)]
mod tests;
