use crate::render_model::ChatUiOptions;
use crate::{
    AccountUsageSnapshot, ChatInputDraft, ChatSession, ChatSettingsReference, ContextUsageSnapshot,
    IconRegistry, ProviderConnectionState, TextCatalog, ThemeTokens, VendorFactRegistry,
    VendorUiState,
};

impl Default for ChatSession {
    fn default() -> Self {
        Self {
            title: "katana-chat-ui".to_string(),
            messages: Vec::new(),
            outputs: Vec::new(),
            draft: ChatInputDraft::new(),
            provider: ProviderConnectionState::missing("provider is not configured"),
            theme: ThemeTokens::default(),
            icons: IconRegistry::default(),
            texts: TextCatalog::english(),
            vendor_facts: VendorFactRegistry::default(),
            vendor_state: VendorUiState::default(),
            context_usage: ContextUsageSnapshot::unavailable(),
            account_usage: AccountUsageSnapshot::unavailable("usage is unavailable"),
            ui_options: ChatUiOptions::new(),
            settings_reference: ChatSettingsReference::default_path(),
            settings_visible: false,
            command_entries: Vec::new(),
            next_message_id: 1,
            next_output_id: 1,
        }
    }
}
