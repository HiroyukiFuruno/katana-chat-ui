use super::model::ChatUiVendorControlSurface;
use crate::{ChatRenderModel, VendorControlItem};

pub(super) struct VendorSurfaceBuilder;

impl VendorSurfaceBuilder {
    pub(super) fn controls(model: &ChatRenderModel) -> Vec<ChatUiVendorControlSurface> {
        let controls = &model.vendor_ui.controls;
        [
            control(&model.texts.endpoint_label, &controls.endpoint),
            control(&model.texts.model_selector, &controls.model),
            control(&model.texts.mode_selector, &controls.mode),
            control(&model.texts.thinking_selector, &controls.thinking),
            control(&model.texts.permission_mode_selector, &controls.permission),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

fn control(label: &str, item: &Option<VendorControlItem>) -> Option<ChatUiVendorControlSurface> {
    let item = item.as_ref()?;
    Some(ChatUiVendorControlSurface {
        key: item.key.clone(),
        label: label.to_string(),
        value: control_value(item),
        options: item.options.clone(),
        enabled: item.enabled,
    })
}

fn control_value(item: &VendorControlItem) -> String {
    match &item.selected_value {
        Some(value) => value.clone(),
        None => item.options.join(", "),
    }
}
