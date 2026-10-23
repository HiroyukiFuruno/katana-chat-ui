use super::model::ChatUiVendorControlSurface;
use crate::{ChatRenderModel, VendorControlItem};

pub(super) struct VendorSurfaceBuilder;

impl VendorSurfaceBuilder {
    pub(super) fn controls(model: &ChatRenderModel) -> Vec<ChatUiVendorControlSurface> {
        let controls = &model.vendor_ui.controls;
        [
            control(&model.texts.endpoint_label, &controls.endpoint, &[]),
            control(
                &model.texts.model_selector,
                &controls.model,
                &model.vendor_ui.model_choices,
            ),
            control(&model.texts.mode_selector, &controls.mode, &[]),
            control(&model.texts.thinking_selector, &controls.thinking, &[]),
            control(
                &model.texts.permission_mode_selector,
                &controls.permission,
                &[],
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

fn control(
    label: &str,
    item: &Option<VendorControlItem>,
    choices: &[crate::VendorUiOption],
) -> Option<ChatUiVendorControlSurface> {
    let item = item.as_ref()?;
    let option_labels = option_labels(&item.options, choices);
    Some(ChatUiVendorControlSurface {
        key: item.key.clone(),
        label: label.to_string(),
        value: control_value(item, choices, &option_labels),
        options: item.options.clone(),
        option_labels,
        enabled: item.enabled,
    })
}

fn control_value(
    item: &VendorControlItem,
    choices: &[crate::VendorUiOption],
    option_labels: &[String],
) -> String {
    match &item.selected_value {
        Some(value) => display_label(value, choices),
        None => option_labels.join(", "),
    }
}

fn option_labels(options: &[String], choices: &[crate::VendorUiOption]) -> Vec<String> {
    options
        .iter()
        .map(|option| display_label(option, choices))
        .collect()
}

fn display_label(value: &str, choices: &[crate::VendorUiOption]) -> String {
    choices
        .iter()
        .find(|choice| choice.id == value)
        .map_or_else(|| value.to_string(), |choice| choice.label.clone())
}
