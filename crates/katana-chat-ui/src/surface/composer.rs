use super::model::{ChatUiActionButtonSurface, ChatUiComposerInputKind, ChatUiComposerSurface};
use crate::{ChatRenderModel, SvgIcon};

struct ComposerButtons {
    attach: ChatUiActionButtonSurface,
    stop: ChatUiActionButtonSurface,
    send: ChatUiActionButtonSurface,
}

impl ComposerButtons {
    fn from_model(model: &ChatRenderModel) -> Self {
        Self {
            attach: action(
                "attach",
                &model.texts.attach_button,
                &model.icons.attach,
                true,
            ),
            stop: action(
                "stop",
                &model.texts.stop_button,
                &model.icons.stop,
                model.input.can_cancel,
            ),
            send: action(
                "send",
                &model.texts.send_button,
                &model.icons.send,
                model.input.can_submit,
            ),
        }
    }
}

impl ChatUiComposerSurface {
    pub(super) fn from_model(model: &ChatRenderModel) -> Self {
        let buttons = ComposerButtons::from_model(model);
        Self {
            text: model.input.text.clone(),
            input_kind: ChatUiComposerInputKind::ImeMultilineEditor,
            ime_enabled: true,
            attachments: model.input.attachments.clone(),
            send_available: model.provider.is_configured(),
            send_enabled: model.input.can_submit,
            stop_enabled: model.input.can_cancel,
            placeholder: model.texts.composer_placeholder.clone(),
            send_label: model.texts.send_button.clone(),
            stop_label: model.texts.stop_button.clone(),
            attach_label: model.texts.attach_button.clone(),
            remove_attachment_label: model.texts.remove_attachment_button.clone(),
            send_icon: model.icons.send.clone(),
            stop_icon: model.icons.stop.clone(),
            attach_icon: model.icons.attach.clone(),
            attach: buttons.attach,
            stop: buttons.stop,
            send: buttons.send,
        }
    }
}

fn action(id: &str, label: &str, icon: &SvgIcon, enabled: bool) -> ChatUiActionButtonSurface {
    ChatUiActionButtonSurface {
        id: id.to_string(),
        label: label.to_string(),
        icon: icon.clone(),
        enabled,
    }
}
