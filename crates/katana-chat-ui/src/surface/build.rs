use super::model::{
    ChatUiActionButtonSurface, ChatUiChromeSurface, ChatUiComposerSurface, ChatUiDebugSurface,
    ChatUiMessageAlignment, ChatUiMessageListSurface, ChatUiMessageSurface,
    ChatUiOutputHandoffSurface, ChatUiOutputSurface, ChatUiSurface, ChatUiThinkingSurface,
    ChatUiUsageSurface, ChatUiVendorBarSurface,
};
use crate::{
    ChatRenderModel, MessageRenderModel, MessageRole, MessageStatus, OutputRenderModel, SvgIcon,
};

impl ChatUiSurface {
    pub fn from_render_model(model: &ChatRenderModel) -> Self {
        let messages = messages(model);
        let composer = ChatUiComposerSurface::from_model(model);
        let vendor_bar = ChatUiVendorBarSurface::from_model(model);
        Self {
            chrome: ChatUiChromeSurface::from_model(model),
            message_list: ChatUiMessageListSurface {
                messages: messages.clone(),
            },
            messages,
            composer,
            vendor_bar,
            usage: ChatUiUsageSurface::from_model(model),
            output_handoff: ChatUiOutputHandoffSurface::from_model(model),
            debug: ChatUiDebugSurface::from_model(model),
            vendor_controls: model.vendor_ui.controls.clone(),
            vendor_ui: model.vendor_ui.clone(),
            settings_icon: model.icons.settings.clone(),
            settings_label: model.texts.settings_button.clone(),
            vendor_selector_label: model.texts.vendor_selector.clone(),
            model_selector_label: model.texts.model_selector.clone(),
            mode_selector_label: model.texts.mode_selector.clone(),
            thinking_selector_label: model.texts.thinking_selector.clone(),
            permission_mode_selector_label: model.texts.permission_mode_selector.clone(),
        }
    }
}

impl ChatUiChromeSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            title: model.title.clone(),
            provider_icon: model.icons.provider.clone(),
            settings: action(
                "settings",
                &model.texts.settings_button,
                &model.icons.settings,
                true,
            ),
        }
    }
}

impl ChatUiMessageSurface {
    fn from_model(model: &MessageRenderModel, chat: &ChatRenderModel) -> Self {
        Self {
            id: model.id,
            role: model.role,
            role_label: role_label(model.role, chat),
            status: model.status.clone(),
            status_label: status_label(&model.status, chat),
            alignment: alignment(model.role),
            body: super::message::MessageSurfaceBuilder::body(model),
            blocks: model.blocks.clone(),
            thinking: model.thinking.as_ref().map(|it| ChatUiThinkingSurface {
                label: it.label.clone(),
                entries: it.entries.clone(),
                expanded: it.expanded,
                completed: it.completed,
            }),
            outputs: outputs_for_message(chat, model.id),
            attachments: model.attachments.clone(),
        }
    }
}

impl ChatUiVendorBarSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        let controls = &model.vendor_ui.controls;
        Self {
            active_vendor_id: controls.active_vendor_id.clone(),
            active_vendor_label: controls.active_vendor_label.clone(),
            vendor_options: controls.vendor_options.clone(),
            vendor_selector_label: model.texts.vendor_selector.clone(),
            controls: super::vendor::VendorSurfaceBuilder::controls(model),
            tools_visible: controls.tools_visible,
            web_search_visible: controls.web_search_visible,
        }
    }
}

impl ChatUiUsageSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            context_percentage: model.context_usage.percentage,
            context_status: format!("{:?}", model.context_usage.status),
            account_label: model.account_usage.unavailable_reason().to_string(),
        }
    }
}

impl ChatUiOutputHandoffSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            label: model.texts.output_handoff.clone(),
            outputs: model
                .outputs
                .iter()
                .map(ChatUiOutputSurface::from_model)
                .collect(),
            json_available: !model.outputs.is_empty(),
        }
    }
}

impl ChatUiDebugSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            enabled: model.ui_options.debug,
            label: model.texts.output_handoff.clone(),
            output_handoff_text: output_handoff_text(model),
        }
    }
}

impl ChatUiOutputSurface {
    fn from_model(model: &OutputRenderModel) -> Self {
        Self {
            id: model.id,
            source_message_id: model.source_message_id,
            kind: model.kind.clone(),
            actions: model.actions.clone(),
        }
    }
}

fn output_handoff_text(model: &ChatRenderModel) -> String {
    match serde_json::to_string_pretty(&model.outputs) {
        Ok(json) => json,
        Err(error) => format!("output JSON generation failed: {error}"),
    }
}

fn messages(model: &ChatRenderModel) -> Vec<ChatUiMessageSurface> {
    model
        .messages
        .iter()
        .map(|it| ChatUiMessageSurface::from_model(it, model))
        .collect()
}

fn outputs_for_message(model: &ChatRenderModel, message_id: u64) -> Vec<ChatUiOutputSurface> {
    model
        .outputs
        .iter()
        .filter(|it| it.source_message_id == message_id)
        .map(ChatUiOutputSurface::from_model)
        .collect()
}

fn action(id: &str, label: &str, icon: &SvgIcon, enabled: bool) -> ChatUiActionButtonSurface {
    ChatUiActionButtonSurface {
        id: id.to_string(),
        label: label.to_string(),
        icon: icon.clone(),
        enabled,
    }
}

fn role_label(role: MessageRole, model: &ChatRenderModel) -> String {
    match role {
        MessageRole::User => model.texts.user_role.clone(),
        MessageRole::Assistant => model.texts.assistant_role.clone(),
        MessageRole::System => model.texts.system_role.clone(),
        MessageRole::Tool => model.texts.tool_role.clone(),
    }
}

fn status_label(status: &MessageStatus, model: &ChatRenderModel) -> String {
    match status {
        MessageStatus::Sending | MessageStatus::Streaming => model.texts.thinking_selector.clone(),
        MessageStatus::Complete => String::new(),
        MessageStatus::Error(error) => error.clone(),
    }
}

fn alignment(role: MessageRole) -> ChatUiMessageAlignment {
    match role {
        MessageRole::User => ChatUiMessageAlignment::Trailing,
        MessageRole::Assistant | MessageRole::System | MessageRole::Tool => {
            ChatUiMessageAlignment::Leading
        }
    }
}
