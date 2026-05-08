use super::{ChatTextKey, TextCatalog};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatTextSet {
    pub locale: String,
    pub composer_placeholder: String,
    pub send_button: String,
    pub stop_button: String,
    pub attach_button: String,
    pub new_chat_button: String,
    pub history_button: String,
    pub settings_button: String,
    pub vendor_selector: String,
    pub model_selector: String,
    pub mode_selector: String,
    pub thinking_selector: String,
    pub permission_mode_selector: String,
    pub user_role: String,
    pub assistant_role: String,
    pub tool_role: String,
    pub system_role: String,
    pub endpoint_label: String,
    pub output_handoff: String,
    pub remove_attachment_button: String,
}

impl ChatTextSet {
    pub fn from_catalog(catalog: &TextCatalog) -> Self {
        Self {
            locale: catalog.locale().code().to_string(),
            composer_placeholder: catalog.resolve(ChatTextKey::ComposerPlaceholder),
            send_button: catalog.resolve(ChatTextKey::SendButton),
            stop_button: catalog.resolve(ChatTextKey::StopButton),
            attach_button: catalog.resolve(ChatTextKey::AttachButton),
            new_chat_button: catalog.resolve(ChatTextKey::NewChatButton),
            history_button: catalog.resolve(ChatTextKey::HistoryButton),
            settings_button: catalog.resolve(ChatTextKey::SettingsButton),
            vendor_selector: catalog.resolve(ChatTextKey::VendorSelector),
            model_selector: catalog.resolve(ChatTextKey::ModelSelector),
            mode_selector: catalog.resolve(ChatTextKey::ModeSelector),
            thinking_selector: catalog.resolve(ChatTextKey::ThinkingSelector),
            permission_mode_selector: catalog.resolve(ChatTextKey::PermissionModeSelector),
            user_role: catalog.resolve(ChatTextKey::UserRole),
            assistant_role: catalog.resolve(ChatTextKey::AssistantRole),
            tool_role: catalog.resolve(ChatTextKey::ToolRole),
            system_role: catalog.resolve(ChatTextKey::SystemRole),
            endpoint_label: catalog.resolve(ChatTextKey::EndpointLabel),
            output_handoff: catalog.resolve(ChatTextKey::OutputHandoff),
            remove_attachment_button: catalog.resolve(ChatTextKey::RemoveAttachmentButton),
        }
    }
}
