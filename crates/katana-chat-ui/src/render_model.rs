use crate::{
    AccountUsageSnapshot, Attachment, ChatMessage, ChatOutput, ChatOutputKind,
    ChatSettingsRenderModel, ChatTextSet, ContextUsageSnapshot, HostActionIntent, IconRegistry,
    MarkdownBlock, MarkdownSubset, MessageRole, MessageStatus, OutputStatus, RoleVisualIntent,
    SlashLauncherRenderModel, SvgIcon, ThemeTokens, VendorUiSurface,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatRenderModel {
    pub title: String,
    pub messages: Vec<MessageRenderModel>,
    pub outputs: Vec<OutputRenderModel>,
    pub input: InputRenderModel,
    pub provider: ProviderConnectionState,
    pub theme: ThemeTokens,
    pub icons: ChatIconSet,
    pub texts: ChatTextSet,
    pub vendor_ui: VendorUiSurface,
    pub context_usage: ContextUsageSnapshot,
    pub account_usage: AccountUsageSnapshot,
    pub ui_options: ChatUiOptions,
    pub settings: ChatSettingsRenderModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRenderModel {
    pub id: u64,
    pub role: MessageRole,
    pub status: MessageStatus,
    pub visual: RoleVisualIntent,
    pub blocks: Vec<MarkdownBlock>,
    pub thinking: Option<ThinkingRenderModel>,
    pub attachments: Vec<Attachment>,
}

impl MessageRenderModel {
    pub fn from_message(message: &ChatMessage) -> Self {
        Self {
            id: message.id,
            role: message.role,
            status: message.status.clone(),
            visual: message.role.visual_intent(),
            blocks: MarkdownSubset::parse(&message.content).blocks,
            thinking: message.thinking.as_ref().map(ThinkingRenderModel::from_log),
            attachments: message.attachments.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingRenderModel {
    pub label: String,
    pub entries: Vec<String>,
    pub expanded: bool,
    pub completed: bool,
}

impl ThinkingRenderModel {
    fn from_log(log: &crate::ThinkingLog) -> Self {
        Self {
            label: log.label.clone(),
            entries: log.entries.clone(),
            expanded: log.expanded,
            completed: log.completed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputRenderModel {
    pub id: u64,
    pub source_message_id: u64,
    pub kind: ChatOutputKind,
    pub status: OutputStatus,
    pub actions: Vec<HostActionIntent>,
}

impl OutputRenderModel {
    pub fn from_output(output: &ChatOutput) -> Self {
        Self {
            id: output.id,
            source_message_id: output.source_message_id,
            kind: output.kind.clone(),
            status: output.status.clone(),
            actions: output.host_actions(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRenderModel {
    pub text: String,
    pub attachments: Vec<Attachment>,
    pub can_submit: bool,
    pub can_cancel: bool,
    pub tray_visible: bool,
    pub slash_launcher: SlashLauncherRenderModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiOptions {
    #[serde(default)]
    pub debug: bool,
}

impl ChatUiOptions {
    pub fn new() -> Self {
        Self { debug: false }
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }
}

impl Default for ChatUiOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderConnectionState {
    Configured(String),
    Missing(String),
}

impl ProviderConnectionState {
    pub fn configured(label: impl Into<String>) -> Self {
        Self::Configured(label.into())
    }

    pub fn missing(reason: impl Into<String>) -> Self {
        Self::Missing(reason.into())
    }

    pub fn is_configured(&self) -> bool {
        matches!(self, Self::Configured(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatIconSet {
    pub send: SvgIcon,
    pub stop: SvgIcon,
    pub attach: SvgIcon,
    pub new_chat: SvgIcon,
    pub history: SvgIcon,
    pub settings: SvgIcon,
    pub provider: SvgIcon,
}

impl ChatIconSet {
    pub fn from_registry(registry: &IconRegistry, active_provider_id: &str) -> Self {
        Self {
            send: registry.resolve("send"),
            stop: registry.resolve("stop"),
            attach: registry.resolve("attach"),
            new_chat: registry.resolve("new-chat"),
            history: registry.resolve("history"),
            settings: registry.resolve("settings"),
            provider: registry.resolve(&format!("provider:{active_provider_id}")),
        }
    }
}
