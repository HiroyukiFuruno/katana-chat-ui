use super::{ChatTextKey, TextCatalogError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TextCatalogOverride {
    pub composer_placeholder: Option<String>,
    pub send_button: Option<String>,
    pub stop_button: Option<String>,
    pub attach_button: Option<String>,
    pub new_chat_button: Option<String>,
    pub history_button: Option<String>,
    pub settings_button: Option<String>,
    pub vendor_selector: Option<String>,
    pub model_selector: Option<String>,
    pub mode_selector: Option<String>,
    pub thinking_selector: Option<String>,
    pub processing_status: Option<String>,
    pub generating_status: Option<String>,
    pub editing_status: Option<String>,
    pub reading_status: Option<String>,
    pub searching_status: Option<String>,
    pub web_searching_status: Option<String>,
    pub executing_status: Option<String>,
    pub permission_mode_selector: Option<String>,
    pub user_role: Option<String>,
    pub assistant_role: Option<String>,
    pub tool_role: Option<String>,
    pub system_role: Option<String>,
    pub endpoint_label: Option<String>,
    pub output_handoff: Option<String>,
    pub remove_attachment_button: Option<String>,
}

impl TextCatalogOverride {
    pub fn from_json(json: &str) -> Result<Self, TextCatalogError> {
        serde_json::from_str(json)
            .map_err(|error| TextCatalogError::InvalidOverrideJson(error.to_string()))
    }

    pub(super) fn resolve(&self, key: ChatTextKey) -> Option<&str> {
        match key {
            ChatTextKey::ComposerPlaceholder => self.composer_placeholder.as_deref(),
            ChatTextKey::SendButton => self.send_button.as_deref(),
            ChatTextKey::StopButton => self.stop_button.as_deref(),
            ChatTextKey::AttachButton => self.attach_button.as_deref(),
            ChatTextKey::NewChatButton => self.new_chat_button.as_deref(),
            ChatTextKey::HistoryButton => self.history_button.as_deref(),
            ChatTextKey::SettingsButton => self.settings_button.as_deref(),
            ChatTextKey::VendorSelector => self.vendor_selector.as_deref(),
            ChatTextKey::ModelSelector => self.model_selector.as_deref(),
            ChatTextKey::ModeSelector => self.mode_selector.as_deref(),
            ChatTextKey::ThinkingSelector => self.thinking_selector.as_deref(),
            ChatTextKey::ProcessingStatus => self.processing_status.as_deref(),
            ChatTextKey::GeneratingStatus => self.generating_status.as_deref(),
            ChatTextKey::EditingStatus => self.editing_status.as_deref(),
            ChatTextKey::ReadingStatus => self.reading_status.as_deref(),
            ChatTextKey::SearchingStatus => self.searching_status.as_deref(),
            ChatTextKey::WebSearchingStatus => self.web_searching_status.as_deref(),
            ChatTextKey::ExecutingStatus => self.executing_status.as_deref(),
            ChatTextKey::PermissionModeSelector => self.permission_mode_selector.as_deref(),
            ChatTextKey::UserRole => self.user_role.as_deref(),
            ChatTextKey::AssistantRole => self.assistant_role.as_deref(),
            ChatTextKey::ToolRole => self.tool_role.as_deref(),
            ChatTextKey::SystemRole => self.system_role.as_deref(),
            ChatTextKey::EndpointLabel => self.endpoint_label.as_deref(),
            ChatTextKey::OutputHandoff => self.output_handoff.as_deref(),
            ChatTextKey::RemoveAttachmentButton => self.remove_attachment_button.as_deref(),
        }
    }

    pub(super) fn set_text(&mut self, key: ChatTextKey, text: String) {
        match key {
            ChatTextKey::ComposerPlaceholder => self.composer_placeholder = Some(text),
            ChatTextKey::SendButton => self.send_button = Some(text),
            ChatTextKey::StopButton => self.stop_button = Some(text),
            ChatTextKey::AttachButton => self.attach_button = Some(text),
            ChatTextKey::NewChatButton => self.new_chat_button = Some(text),
            ChatTextKey::HistoryButton => self.history_button = Some(text),
            ChatTextKey::SettingsButton => self.settings_button = Some(text),
            ChatTextKey::VendorSelector => self.vendor_selector = Some(text),
            ChatTextKey::ModelSelector => self.model_selector = Some(text),
            ChatTextKey::ModeSelector => self.mode_selector = Some(text),
            ChatTextKey::ThinkingSelector => self.thinking_selector = Some(text),
            ChatTextKey::ProcessingStatus => self.processing_status = Some(text),
            ChatTextKey::GeneratingStatus => self.generating_status = Some(text),
            ChatTextKey::EditingStatus => self.editing_status = Some(text),
            ChatTextKey::ReadingStatus => self.reading_status = Some(text),
            ChatTextKey::SearchingStatus => self.searching_status = Some(text),
            ChatTextKey::WebSearchingStatus => self.web_searching_status = Some(text),
            ChatTextKey::ExecutingStatus => self.executing_status = Some(text),
            ChatTextKey::PermissionModeSelector => self.permission_mode_selector = Some(text),
            ChatTextKey::UserRole => self.user_role = Some(text),
            ChatTextKey::AssistantRole => self.assistant_role = Some(text),
            ChatTextKey::ToolRole => self.tool_role = Some(text),
            ChatTextKey::SystemRole => self.system_role = Some(text),
            ChatTextKey::EndpointLabel => self.endpoint_label = Some(text),
            ChatTextKey::OutputHandoff => self.output_handoff = Some(text),
            ChatTextKey::RemoveAttachmentButton => self.remove_attachment_button = Some(text),
        }
    }

    pub(super) fn merge(&mut self, next: Self) {
        self.merge_composer_controls(&next);
        self.merge_provider_controls(&next);
        self.merge_activity_labels(&next);
        self.merge_role_and_handoff_labels(next);
    }

    fn merge_composer_controls(&mut self, next: &Self) {
        self.merge_key(
            ChatTextKey::ComposerPlaceholder,
            next.composer_placeholder.clone(),
        );
        self.merge_key(ChatTextKey::SendButton, next.send_button.clone());
        self.merge_key(ChatTextKey::StopButton, next.stop_button.clone());
        self.merge_key(ChatTextKey::AttachButton, next.attach_button.clone());
        self.merge_key(ChatTextKey::NewChatButton, next.new_chat_button.clone());
        self.merge_key(ChatTextKey::HistoryButton, next.history_button.clone());
        self.merge_key(ChatTextKey::SettingsButton, next.settings_button.clone());
    }

    fn merge_provider_controls(&mut self, next: &Self) {
        self.merge_key(ChatTextKey::VendorSelector, next.vendor_selector.clone());
        self.merge_key(ChatTextKey::ModelSelector, next.model_selector.clone());
        self.merge_key(ChatTextKey::ModeSelector, next.mode_selector.clone());
        self.merge_key(
            ChatTextKey::ThinkingSelector,
            next.thinking_selector.clone(),
        );
        self.merge_key(
            ChatTextKey::PermissionModeSelector,
            next.permission_mode_selector.clone(),
        );
    }

    fn merge_activity_labels(&mut self, next: &Self) {
        self.merge_key(
            ChatTextKey::ProcessingStatus,
            next.processing_status.clone(),
        );
        self.merge_key(
            ChatTextKey::GeneratingStatus,
            next.generating_status.clone(),
        );
        self.merge_key(ChatTextKey::EditingStatus, next.editing_status.clone());
        self.merge_key(ChatTextKey::ReadingStatus, next.reading_status.clone());
        self.merge_key(ChatTextKey::SearchingStatus, next.searching_status.clone());
        self.merge_key(
            ChatTextKey::WebSearchingStatus,
            next.web_searching_status.clone(),
        );
        self.merge_key(ChatTextKey::ExecutingStatus, next.executing_status.clone());
    }

    fn merge_role_and_handoff_labels(&mut self, next: Self) {
        self.merge_key(ChatTextKey::UserRole, next.user_role);
        self.merge_key(ChatTextKey::AssistantRole, next.assistant_role);
        self.merge_key(ChatTextKey::ToolRole, next.tool_role);
        self.merge_key(ChatTextKey::SystemRole, next.system_role);
        self.merge_key(ChatTextKey::EndpointLabel, next.endpoint_label);
        self.merge_key(ChatTextKey::OutputHandoff, next.output_handoff);
        self.merge_key(
            ChatTextKey::RemoveAttachmentButton,
            next.remove_attachment_button,
        );
    }

    fn merge_key(&mut self, key: ChatTextKey, value: Option<String>) {
        if let Some(text) = value {
            self.set_text(key, text);
        }
    }
}
