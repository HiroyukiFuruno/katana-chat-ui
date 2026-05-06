use crate::{
    ChatRenderModel, ChatSession, InputRenderModel, MessageRenderModel, OutputRenderModel,
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
        }
    }

    fn input_render_model(&self) -> InputRenderModel {
        InputRenderModel {
            text: self.draft.text.clone(),
            attachments: self.draft.attachments.clone(),
            can_submit: self.provider.is_configured() && self.draft.can_submit(),
            can_cancel: self.has_streaming_message(),
            tray_visible: !self.draft.attachments.is_empty(),
        }
    }
}
