use crate::vendor_ui::{VendorFactRegistry, VendorUiProfile, VendorUiState};
use crate::{
    AccountUsageSnapshot, ChatInputDraft, ChatMessage, ChatOutput, ChatSettingsError,
    ChatSettingsReference, CommandLaunchEntry, ContextUsageSnapshot, IconRegistry, MessageRole,
    ProviderConnectionState, SvgIcon, TextCatalog, ThemeTokens, VendorUiCapabilities,
};
use crate::{config::ChatUiConfig, render_model::ChatUiOptions};

mod agent_ops;
mod defaults;
mod error;
mod output_ops;
mod render_ops;
mod snapshot;
mod stream_ops;
mod text_ops;

pub use error::ChatSessionError;
pub use snapshot::ChatSessionSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSession {
    title: String,
    messages: Vec<ChatMessage>,
    outputs: Vec<ChatOutput>,
    draft: ChatInputDraft,
    provider: ProviderConnectionState,
    theme: ThemeTokens,
    icons: IconRegistry,
    texts: TextCatalog,
    vendor_facts: VendorFactRegistry,
    vendor_state: VendorUiState,
    context_usage: ContextUsageSnapshot,
    account_usage: AccountUsageSnapshot,
    ui_options: ChatUiOptions,
    settings_reference: ChatSettingsReference,
    settings_visible: bool,
    command_entries: Vec<CommandLaunchEntry>,
    next_message_id: u64,
    next_output_id: u64,
}

impl ChatSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ChatUiConfig) -> Result<Self, ChatSettingsError> {
        let mut session = Self::default();
        session.apply_config(config)?;
        Ok(session)
    }

    pub fn apply_config(&mut self, config: ChatUiConfig) -> Result<(), ChatSettingsError> {
        let reference = config.settings_reference()?;
        self.settings_reference = reference;
        self.ui_options = config.options;
        Ok(())
    }

    pub fn draft_mut(&mut self) -> &mut ChatInputDraft {
        &mut self.draft
    }

    pub fn draft(&self) -> &ChatInputDraft {
        &self.draft
    }

    pub fn set_provider_configured(&mut self, label: impl Into<String>) {
        self.provider = ProviderConnectionState::configured(label);
    }

    pub fn set_provider_missing(&mut self, reason: impl Into<String>) {
        self.provider = ProviderConnectionState::missing(reason);
    }

    pub fn set_context_usage(&mut self, usage: ContextUsageSnapshot) {
        self.context_usage = usage;
    }

    pub fn set_account_usage(&mut self, usage: AccountUsageSnapshot) {
        self.account_usage = usage;
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_text_catalog(&mut self, catalog: TextCatalog) {
        self.texts = catalog;
    }

    pub fn set_vendor_ui_capabilities(&mut self, capabilities: VendorUiCapabilities) {
        self.set_vendor_ui_profile(VendorUiProfile::new("default", "Default", capabilities));
    }

    pub fn set_vendor_ui_profile(&mut self, profile: VendorUiProfile) {
        self.vendor_state = VendorUiState::from_profile(&profile);
    }

    pub fn set_vendor_ui_state(&mut self, state: VendorUiState) {
        self.vendor_state = state;
    }

    pub fn vendor_ui_state(&self) -> &VendorUiState {
        &self.vendor_state
    }

    pub fn set_vendor_fact_registry(&mut self, registry: VendorFactRegistry) {
        self.vendor_facts = registry;
    }

    pub fn override_icon(&mut self, icon: SvgIcon) {
        self.icons.override_icon(icon);
    }

    pub fn open_settings(&mut self) {
        self.settings_visible = true;
    }

    pub fn toggle_settings(&mut self) {
        self.settings_visible = !self.settings_visible;
    }

    pub fn close_settings(&mut self) {
        self.settings_visible = false;
    }

    pub fn set_command_entries(&mut self, entries: Vec<CommandLaunchEntry>) {
        self.command_entries = entries;
    }

    pub fn submit_draft(&mut self) -> Result<u64, ChatSessionError> {
        self.ensure_provider_configured()?;
        if !self.draft.can_submit() {
            return Err(ChatSessionError::DraftEmpty);
        }
        let message_id = self.next_message_id();
        let attachments = self.draft.take_attachments();
        let message = ChatMessage::new(message_id, MessageRole::User, self.draft.text.clone())
            .with_attachments(attachments);
        self.messages.push(message);
        self.draft.clear();
        Ok(message_id)
    }

    fn ensure_provider_configured(&self) -> Result<(), ChatSessionError> {
        if self.provider.is_configured() {
            return Ok(());
        }
        Err(ChatSessionError::ProviderMissing)
    }

    fn next_message_id(&mut self) -> u64 {
        let message_id = self.next_message_id;
        self.next_message_id = self.next_message_id.saturating_add(1);
        message_id
    }
}

#[cfg(test)]
mod agent_ops_tests;
#[cfg(test)]
mod tests;
