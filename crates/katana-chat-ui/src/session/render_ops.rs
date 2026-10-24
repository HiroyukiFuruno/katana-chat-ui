use crate::{
    ChatRenderModel, ChatSession, ChatSettingsItem, ChatSettingsRenderModel, ChatSettingsSection,
    InputRenderModel, MessageRenderModel, OutputRenderModel, SlashLauncherRenderModel,
    VendorUiSurface,
};

impl ChatSession {
    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.ui_options = self.ui_options.clone().with_debug(enabled);
    }

    pub fn render_model(&self) -> ChatRenderModel {
        ChatRenderModel {
            title: self.title.clone(),
            messages: self
                .messages
                .iter()
                .map(MessageRenderModel::from_message)
                .collect(),
            outputs: self
                .outputs
                .iter()
                .map(OutputRenderModel::from_output)
                .collect(),
            input: self.input_render_model(),
            provider: self.provider.clone(),
            theme: self.theme.clone(),
            icons: crate::render_model::ChatIconSet::from_registry(
                &self.icons,
                &self.vendor_state.active_vendor_id,
            ),
            texts: crate::text::ChatTextSet::from_catalog(&self.texts),
            vendor_ui: VendorUiSurface::from_registry(&self.vendor_facts, &self.vendor_state),
            context_usage: self.context_usage.clone(),
            account_usage: self.account_usage.clone(),
            ui_options: self.ui_options.clone(),
            settings: self.settings_render_model(),
        }
    }

    fn input_render_model(&self) -> InputRenderModel {
        InputRenderModel {
            text: self.draft.text.clone(),
            attachments: self.draft.attachments.clone(),
            can_submit: self.provider.is_configured() && self.draft.can_submit(),
            can_cancel: self.has_streaming_message(),
            tray_visible: !self.draft.attachments.is_empty(),
            slash_launcher: SlashLauncherRenderModel::from_draft(
                &self.draft.text,
                &self.command_entries,
            ),
        }
    }

    fn settings_render_model(&self) -> ChatSettingsRenderModel {
        ChatSettingsRenderModel {
            visible: self.settings_visible,
            reference: self.settings_reference.clone(),
            sections: self.settings_sections(),
        }
    }

    fn settings_sections(&self) -> Vec<ChatSettingsSection> {
        vec![
            self.settings_section("appearance", "Appearance", self.appearance_items()),
            self.settings_section("provider", "Provider", self.provider_items()),
            self.settings_section("composer", "Composer", self.composer_items()),
        ]
    }

    fn settings_section(
        &self,
        id: &str,
        label: &str,
        items: Vec<ChatSettingsItem>,
    ) -> ChatSettingsSection {
        ChatSettingsSection {
            id: id.to_string(),
            label: label.to_string(),
            items,
        }
    }

    fn appearance_items(&self) -> Vec<ChatSettingsItem> {
        vec![
            self.settings_item("theme", "Theme", "default"),
            self.settings_item("locale", "Language", self.texts.locale().code()),
            self.settings_item(
                "placeholder",
                "Placeholder",
                &self.texts.resolve(crate::ChatTextKey::ComposerPlaceholder),
            ),
            self.settings_item("icons", "SVG icons", "runtime overrides"),
        ]
    }

    fn provider_items(&self) -> Vec<ChatSettingsItem> {
        vec![
            self.settings_item(
                "active-provider",
                "Active provider",
                &self.vendor_state.active_vendor_id,
            ),
            self.settings_item("provider-order", "Provider order", "host detected order"),
        ]
    }

    fn composer_items(&self) -> Vec<ChatSettingsItem> {
        vec![
            self.settings_item("submit-key", "Submit key", "Command+Enter"),
            self.settings_item("enter-key", "Enter key", "newline"),
            self.settings_item("slash-launcher", "Slash launcher", "enabled"),
        ]
    }

    fn settings_item(&self, id: &str, label: &str, value: &str) -> ChatSettingsItem {
        ChatSettingsItem {
            id: id.to_string(),
            label: label.to_string(),
            value: value.to_string(),
        }
    }
}
