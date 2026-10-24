use super::{ChatUiActionError, node};
use crate::ChatUiVendorBarSurface;
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiDisplay, UiInteractionState, UiLayoutAxis, UiNode, UiNodeKind,
};

pub(super) const CONTROL_WIDTH: u16 = 132;
pub(super) const CONTROL_HEIGHT: u16 = 20;
pub(super) const CONTROL_GAP: u16 = 8;

mod dropdown;
pub(super) use dropdown::{OPTION_STEP, options};

pub(super) fn selector(
    surface: &ChatUiVendorBarSurface,
    open_dropdown: Option<&str>,
) -> Result<UiNode, ChatUiActionError> {
    Ok(selector_node(surface, open_dropdown))
}

fn selector_node(surface: &ChatUiVendorBarSurface, open_dropdown: Option<&str>) -> UiNode {
    node::container(
        UiNodeKind::SelectBox,
        &surface.active_vendor_label,
        "chat.chrome.provider-selector",
    )
    .common(UiCommonProps::default().width(UiDimension::px(CONTROL_WIDTH)))
    .interaction(UiInteractionState {
        open: open_dropdown == Some("chat.chrome.provider-selector"),
        value: selector_value(surface),
        item_count: surface.vendor_options.len(),
        ..UiInteractionState::default()
    })
    .child(node::text(
        &surface.vendor_selector_label,
        "chat.chrome.provider-selector.label",
        "provider-selector-label",
    ))
}

fn selector_value(surface: &ChatUiVendorBarSurface) -> String {
    format!("{} ▾", surface.active_vendor_label)
}

pub(super) fn controls(
    surface: &ChatUiVendorBarSurface,
    open_dropdown: Option<&str>,
    available_width: u16,
) -> Result<UiNode, ChatUiActionError> {
    let mut controls = control_container(surface.controls.len(), available_width);
    let width = control_width(available_width);
    let kind = control_kind(available_width);
    for (index, control) in surface.controls.iter().enumerate() {
        controls = controls.child(control_node(control, index, open_dropdown, width, kind)?);
    }
    Ok(controls)
}

pub(super) fn control_width(available_width: u16) -> u16 {
    available_width.clamp(1, CONTROL_WIDTH)
}

fn control_kind(available_width: u16) -> UiNodeKind {
    if available_width < CONTROL_WIDTH {
        UiNodeKind::TextButton
    } else {
        UiNodeKind::ComboBox
    }
}

pub(super) fn controls_are_stacked(control_count: usize, available_width: u16) -> bool {
    controls_width(control_count, CONTROL_WIDTH) > available_width
}

pub(super) fn controls_height(control_count: usize, available_width: u16) -> u16 {
    if controls_are_stacked(control_count, available_width) {
        stacked_height(control_count)
    } else {
        CONTROL_HEIGHT
    }
}

fn control_container(control_count: usize, available_width: u16) -> UiNode {
    let stacked = controls_are_stacked(control_count, available_width);
    let (kind, layout_axis) = if stacked {
        (UiNodeKind::Column, UiLayoutAxis::Vertical)
    } else {
        (UiNodeKind::Row, UiLayoutAxis::Horizontal)
    };
    node::container(kind, "Provider controls", "chat.composer.controls").common(
        UiCommonProps::default()
            .display(UiDisplay::Flex)
            .layout_axis(layout_axis)
            .gap(UiDimension::px(CONTROL_GAP)),
    )
}

fn control_node(
    control: &crate::ChatUiVendorControlSurface,
    control_index: usize,
    open_dropdown: Option<&str>,
    width: u16,
    kind: UiNodeKind,
) -> Result<UiNode, ChatUiActionError> {
    let node_id = format!("chat.composer.control.{control_index}");
    let node = node::container(kind, &control.label, &node_id)
        .common(UiCommonProps::default().width(UiDimension::px(width)))
        .interaction(UiInteractionState {
            open: open_dropdown == Some(node_id.as_str()),
            value: control.value.clone(),
            item_count: control.options.len(),
            ..UiInteractionState::default()
        })
        .disabled(!control.enabled);
    Ok(node)
}

fn controls_width(control_count: usize, width: u16) -> u16 {
    let count = u16::try_from(control_count).unwrap_or(u16::MAX);
    let gaps = count.saturating_sub(1).saturating_mul(CONTROL_GAP);
    count.saturating_mul(width).saturating_add(gaps)
}

fn stacked_height(control_count: usize) -> u16 {
    controls_width(control_count, CONTROL_HEIGHT)
}

#[cfg(test)]
#[path = "vendor/tests.rs"]
mod tests;
