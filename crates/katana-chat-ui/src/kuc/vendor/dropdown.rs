use super::{ChatUiActionError, node};
use crate::{ChatUiAction, ChatUiVendorBarSurface, ChatUiVendorControlSurface};
use katana_ui_core::render_model::{UiDimension, UiNode, UiNodeKind};

pub(in crate::kuc) const OPTION_STEP: u16 = 28;
const PROVIDER_SELECTOR: &str = "chat.chrome.provider-selector";
const CONTROL_PREFIX: &str = "chat.composer.control.";

pub(in crate::kuc) fn options(
    surface: &ChatUiVendorBarSurface,
    dropdown_id: &str,
    width: u16,
) -> Result<Vec<UiNode>, ChatUiActionError> {
    if dropdown_id == PROVIDER_SELECTOR {
        return provider_options(surface, width);
    }
    control_index(dropdown_id).map_or_else(
        || Ok(Vec::new()),
        |index| control_options(surface, index, width),
    )
}

fn provider_options(
    surface: &ChatUiVendorBarSurface,
    width: u16,
) -> Result<Vec<UiNode>, ChatUiActionError> {
    surface
        .vendor_options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            option_node(
                &option.label,
                &format!("chat.chrome.provider-option.{index}"),
                ChatUiAction::SelectVendor {
                    vendor_id: option.id.clone(),
                },
                true,
                width,
            )
        })
        .collect()
}

fn control_options(
    surface: &ChatUiVendorBarSurface,
    control_index: usize,
    width: u16,
) -> Result<Vec<UiNode>, ChatUiActionError> {
    let Some(control) = surface.controls.get(control_index) else {
        return Ok(Vec::new());
    };
    control
        .options
        .iter()
        .enumerate()
        .map(|(option_index, value)| {
            control_option(control, control_index, option_index, value, width)
        })
        .collect()
}

fn control_option(
    control: &ChatUiVendorControlSurface,
    control_index: usize,
    option_index: usize,
    value: &str,
    width: u16,
) -> Result<UiNode, ChatUiActionError> {
    let label = control
        .option_labels
        .get(option_index)
        .map_or(value, String::as_str);
    option_node(
        label,
        &format!("chat.composer.control.{control_index}.option.{option_index}"),
        ChatUiAction::SelectVendorControl {
            key: control.key.clone(),
            value: value.to_string(),
        },
        control.enabled,
        width,
    )
}

fn option_node(
    label: &str,
    node_id: &str,
    action: ChatUiAction,
    enabled: bool,
    width: u16,
) -> Result<UiNode, ChatUiActionError> {
    node::action(UiNodeKind::Button, label, node_id, &action)
        .map(|node| node.width(UiDimension::px(width)).disabled(!enabled))
}

fn control_index(dropdown_id: &str) -> Option<usize> {
    dropdown_id
        .strip_prefix(CONTROL_PREFIX)?
        .parse::<usize>()
        .ok()
}
