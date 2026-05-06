use katana_chat_ui::{Attachment, InputRenderModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerView {
    pub text: String,
    pub attachments: Vec<Attachment>,
    pub multiline_enabled: bool,
    pub ime_enabled: bool,
    pub send_enabled: bool,
    pub stop_enabled: bool,
    pub tray_visible: bool,
}

impl ComposerView {
    pub fn from_model(model: &InputRenderModel) -> Self {
        Self {
            text: model.text.clone(),
            attachments: model.attachments.clone(),
            multiline_enabled: true,
            ime_enabled: true,
            send_enabled: model.can_submit,
            stop_enabled: model.can_cancel,
            tray_visible: model.tray_visible,
        }
    }
}
