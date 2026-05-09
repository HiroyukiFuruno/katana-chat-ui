use crate::{
    Attachment, ChatOutputKind, CommandLaunchEntry, HostActionIntent, MarkdownBlock, MessageRole,
    MessageStatus, SvgIcon, VendorControlRenderModel, VendorOption, VendorUiSurface,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiSurface {
    pub chrome: ChatUiChromeSurface,
    pub history_panel: ChatUiHistoryPanelSurface,
    pub message_list: ChatUiMessageListSurface,
    pub messages: Vec<ChatUiMessageSurface>,
    pub composer: ChatUiComposerSurface,
    pub vendor_bar: ChatUiVendorBarSurface,
    pub usage: ChatUiUsageSurface,
    pub output_handoff: ChatUiOutputHandoffSurface,
    pub vendor_ui: VendorUiSurface,
    pub vendor_controls: VendorControlRenderModel,
    pub vendor_selector_label: String,
    pub model_selector_label: String,
    pub mode_selector_label: String,
    pub thinking_selector_label: String,
    pub permission_mode_selector_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiChromeSurface {
    pub title: String,
    pub provider_icon: SvgIcon,
    pub new_chat: ChatUiActionButtonSurface,
    pub history: ChatUiActionButtonSurface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiHistoryPanelSurface {
    pub visible: bool,
    pub label: String,
    pub empty_label: String,
    pub sessions: Vec<ChatUiHistorySessionSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiHistorySessionSurface {
    pub session_id: String,
    pub title: String,
    pub provider_id: String,
    pub provider_label: String,
    pub updated_at_label: String,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiMessageListSurface {
    pub messages: Vec<ChatUiMessageSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiMessageSurface {
    pub id: u64,
    pub role: MessageRole,
    pub role_label: String,
    pub status: MessageStatus,
    pub status_label: String,
    pub alignment: ChatUiMessageAlignment,
    pub body: String,
    pub blocks: Vec<MarkdownBlock>,
    pub thinking: Option<ChatUiThinkingSurface>,
    pub outputs: Vec<ChatUiOutputSurface>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiThinkingSurface {
    pub label: String,
    pub entries: Vec<String>,
    pub expanded: bool,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatUiMessageAlignment {
    Leading,
    Trailing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiComposerSurface {
    pub text: String,
    pub input_kind: ChatUiComposerInputKind,
    pub ime_enabled: bool,
    pub attachments: Vec<Attachment>,
    pub send_available: bool,
    pub send_enabled: bool,
    pub stop_enabled: bool,
    pub placeholder: String,
    pub send_label: String,
    pub stop_label: String,
    pub attach_label: String,
    pub remove_attachment_label: String,
    pub slash_launcher: ChatUiSlashLauncherSurface,
    pub send_icon: SvgIcon,
    pub stop_icon: SvgIcon,
    pub attach_icon: SvgIcon,
    pub attach: ChatUiActionButtonSurface,
    pub stop: ChatUiActionButtonSurface,
    pub send: ChatUiActionButtonSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatUiComposerInputKind {
    ImeMultilineEditor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiActionButtonSurface {
    pub id: String,
    pub label: String,
    pub icon: SvgIcon,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiVendorBarSurface {
    pub active_vendor_id: String,
    pub active_vendor_label: String,
    pub vendor_options: Vec<VendorOption>,
    pub vendor_selector_label: String,
    pub controls: Vec<ChatUiVendorControlSurface>,
    pub tools_visible: bool,
    pub web_search_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiVendorControlSurface {
    pub key: String,
    pub label: String,
    pub value: String,
    pub options: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiUsageSurface {
    pub used_tokens: u64,
    pub max_tokens: u64,
    pub context_percentage: u8,
    pub context_status: String,
    pub account_label: String,
}

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
    pub actions: Vec<HostActionIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiSlashLauncherSurface {
    pub visible: bool,
    pub query: String,
    pub entries: Vec<CommandLaunchEntry>,
}
