use super::{
    labels,
    model::{
        ChatUiActionButtonSurface, ChatUiActivitySurface, ChatUiChromeSurface,
        ChatUiComposerSurface, ChatUiHistoryPanelSurface, ChatUiMessageListSurface,
        ChatUiMessageSurface, ChatUiOutputHandoffSurface, ChatUiOutputSurface, ChatUiSurface,
        ChatUiThinkingSurface, ChatUiUsageSurface, ChatUiVendorBarSurface,
    },
};
use crate::{ChatRenderModel, MessageRenderModel, OutputRenderModel, SvgIcon};

impl ChatUiSurface {
    pub fn from_render_model(model: &ChatRenderModel) -> Self {
        let messages = messages(model);
        let composer = ChatUiComposerSurface::from_model(model);
        let vendor_bar = ChatUiVendorBarSurface::from_model(model);
        Self {
            chrome: ChatUiChromeSurface::from_model(model),
            history_panel: ChatUiHistoryPanelSurface::from_model(model),
            message_list: ChatUiMessageListSurface {
                messages: messages.clone(),
            },
            messages,
            composer,
            vendor_bar,
            usage: ChatUiUsageSurface::from_model(model),
            output_handoff: ChatUiOutputHandoffSurface::from_model(model),
            vendor_controls: model.vendor_ui.controls.clone(),
            vendor_ui: model.vendor_ui.clone(),
            vendor_selector_label: model.texts.vendor_selector.clone(),
            model_selector_label: model.texts.model_selector.clone(),
            mode_selector_label: model.texts.mode_selector.clone(),
            thinking_selector_label: model.texts.thinking_selector.clone(),
            permission_mode_selector_label: model.texts.permission_mode_selector.clone(),
        }
    }
}

impl ChatUiHistoryPanelSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            visible: false,
            label: model.texts.history_button.clone(),
            empty_label: "No history".to_string(),
            sessions: Vec::new(),
        }
    }
}

impl ChatUiChromeSurface {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            title: model.title.clone(),
            provider_icon: model.icons.provider.clone(),
            new_chat: action(
                "new-chat",
                &model.texts.new_chat_button,
                &model.icons.new_chat,
                true,
            ),
            history: action(
                "history",
                &model.texts.history_button,
                &model.icons.history,
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
            role_label: labels::MessageLabelBuilder::role_label(model.role, chat),
            status: model.status.clone(),
            status_label: labels::MessageLabelBuilder::status_label(&model.status, chat),
            activity: model.activity.as_ref().map(|it| ChatUiActivitySurface {
                kind: it.kind,
                label: labels::MessageLabelBuilder::activity_label(it.kind, chat),
            }),
            alignment: labels::MessageLabelBuilder::alignment(model.role),
            body: super::message::MessageSurfaceBuilder::body(model),
            blocks: model.blocks.clone(),
            thinking: model.thinking.as_ref().map(|it| ChatUiThinkingSurface {
                label: it.label.clone(),
                entries: it.entries.clone(),
                expanded: it.expanded,
                completed: it.completed,
            }),
            outputs: Vec::new(),
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
            used_tokens: model.context_usage.used_tokens,
            max_tokens: model.context_usage.max_tokens,
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

fn messages(model: &ChatRenderModel) -> Vec<ChatUiMessageSurface> {
    model
        .messages
        .iter()
        .map(|it| ChatUiMessageSurface::from_model(it, model))
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
