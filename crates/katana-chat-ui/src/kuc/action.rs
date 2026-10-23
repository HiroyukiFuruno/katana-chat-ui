use crate::{CommandLaunchKind, HostActionKind};
use katana_ui_core::render_model::{UiHostActionKind, UiHostActionPlan, UiHostActionSpec};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CHAT_UI_ACTION_ID: &str = "katana-chat-ui.action";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatUiAction {
    Attach,
    RemoveAttachment {
        index: usize,
    },
    NewChat,
    ToggleHistory,
    SelectHistory {
        session_id: String,
    },
    Submit,
    Stop,
    SelectVendor {
        vendor_id: String,
    },
    SelectVendorControl {
        key: String,
        value: String,
    },
    RespondPermission {
        request_id: u64,
        option_id: Option<String>,
    },
    SelectSlashCommand {
        entry_id: String,
        kind: CommandLaunchKind,
    },
    Output {
        output_id: u64,
        kind: HostActionKind,
    },
}

#[derive(Debug, Error)]
pub enum ChatUiActionError {
    #[error("unsupported KUC host action `{0}`")]
    Unsupported(String),
    #[error("invalid KUC chat action payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
}

pub struct ChatUiActionMapper;

impl ChatUiActionMapper {
    pub fn host_action(
        action: &ChatUiAction,
        label: &str,
    ) -> Result<UiHostActionSpec, ChatUiActionError> {
        Ok(
            UiHostActionSpec::new(UiHostActionKind::Custom, CHAT_UI_ACTION_ID, label)
                .payload(serde_json::to_string(action)?),
        )
    }

    pub fn from_host_action(action: &UiHostActionPlan) -> Result<ChatUiAction, ChatUiActionError> {
        if action.action_id != CHAT_UI_ACTION_ID || action.kind != UiHostActionKind::Custom {
            return Err(ChatUiActionError::Unsupported(action.action_id.clone()));
        }
        serde_json::from_str(&action.payload).map_err(ChatUiActionError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::{CHAT_UI_ACTION_ID, ChatUiAction, ChatUiActionError, ChatUiActionMapper};
    use crate::HostActionKind;
    use katana_ui_core::render_model::{UiHostActionKind, UiHostActionPlan, UiNodeId};

    #[test]
    fn host_action_round_trips_typed_output_action() -> Result<(), Box<dyn std::error::Error>> {
        let source = ChatUiAction::Output {
            output_id: 42,
            kind: HostActionKind::ApplyDiff,
        };
        let plan = UiHostActionPlan::new(
            UiNodeId::new("chat.output.42.apply-diff"),
            ChatUiActionMapper::host_action(&source, "Apply diff")?,
        );

        assert_eq!(CHAT_UI_ACTION_ID, plan.action_id);
        assert_eq!(source, ChatUiActionMapper::from_host_action(&plan)?);
        Ok(())
    }

    #[test]
    fn decoder_rejects_wrong_action_id_kind_and_payload() -> Result<(), Box<dyn std::error::Error>>
    {
        let action = ChatUiAction::Submit;
        let plan = UiHostActionPlan::new(
            UiNodeId::new("chat.composer.submit"),
            ChatUiActionMapper::host_action(&action, "Send")?,
        );

        let mut wrong_id = plan.clone();
        wrong_id.action_id = "other.action".to_string();
        assert!(matches!(
            ChatUiActionMapper::from_host_action(&wrong_id),
            Err(ChatUiActionError::Unsupported(_))
        ));

        let mut wrong_kind = plan.clone();
        wrong_kind.kind = UiHostActionKind::Command;
        assert!(matches!(
            ChatUiActionMapper::from_host_action(&wrong_kind),
            Err(ChatUiActionError::Unsupported(_))
        ));

        let mut invalid_payload = plan;
        invalid_payload.payload = "not-json".to_string();
        assert!(matches!(
            ChatUiActionMapper::from_host_action(&invalid_payload),
            Err(ChatUiActionError::InvalidPayload(_))
        ));
        Ok(())
    }
}
