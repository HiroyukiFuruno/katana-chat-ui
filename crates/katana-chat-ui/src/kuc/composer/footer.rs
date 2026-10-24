use super::super::{ChatUiAction, ChatUiActionError, node};
use crate::{ChatUiComposerSurface, ChatUiUsageSurface, ChatUiVendorBarSurface};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiEdgeInsets, UiIconProps, UiNode, UiNodeKind, UiPosition,
    UiVariant,
};

const ACTION_WIDTH: u16 = 32;
const ACTION_HEIGHT: u16 = 28;
const USAGE_WIDTH: u16 = 28;
const FOOTER_GAP: u16 = 4;

pub(super) fn footer(
    surface: &ChatUiComposerSurface,
    vendor: &ChatUiVendorBarSurface,
    usage_surface: &ChatUiUsageSurface,
    open_dropdown: Option<&str>,
    available_width: u16,
) -> Result<UiNode, ChatUiActionError> {
    Ok(
        node::container(UiNodeKind::Column, "", "chat.composer.footer")
            .common(UiCommonProps::default().gap(UiDimension::px(FOOTER_GAP)))
            .child(super::super::vendor::controls(
                vendor,
                open_dropdown,
                available_width,
            )?)
            .child(action_row(surface, usage_surface, available_width)?),
    )
}

pub(super) fn height(control_count: usize, available_width: u16) -> u16 {
    super::super::vendor::controls_height(control_count, available_width)
        .saturating_add(FOOTER_GAP)
        .saturating_add(ACTION_HEIGHT)
}

fn action_row(
    surface: &ChatUiComposerSurface,
    usage_surface: &ChatUiUsageSurface,
    available_width: u16,
) -> Result<UiNode, ChatUiActionError> {
    Ok(
        node::container(UiNodeKind::Stack, "", "chat.composer.actions")
            .common(
                UiCommonProps::default()
                    .width(UiDimension::px(available_width))
                    .height(UiDimension::px(ACTION_HEIGHT)),
            )
            .child(at_x(
                usage(usage_surface),
                trailing_left(available_width, ACTION_WIDTH + USAGE_WIDTH),
            ))
            .child(at_x(
                primary_action(surface)?,
                trailing_left(available_width, ACTION_WIDTH),
            )),
    )
}

fn at_x(node: UiNode, left: u16) -> UiNode {
    node.position(UiPosition::Absolute).margin(insets(left, 0))
}

fn trailing_left(available_width: u16, occupied_width: u16) -> u16 {
    available_width.saturating_sub(occupied_width)
}

fn insets(left: u16, right: u16) -> UiEdgeInsets {
    UiEdgeInsets {
        top: UiDimension::px(0),
        right: UiDimension::px(right),
        bottom: UiDimension::px(0),
        left: UiDimension::px(left),
    }
}

fn primary_action(surface: &ChatUiComposerSurface) -> Result<UiNode, ChatUiActionError> {
    if surface.stop_enabled {
        return icon_action(
            &surface.stop,
            "chat.composer.stop",
            ChatUiAction::Stop,
            surface.stop.enabled,
        );
    }
    icon_action(
        &surface.send,
        "chat.composer.submit",
        ChatUiAction::Submit,
        surface.send_available && surface.send_enabled && surface.send.enabled,
    )
}

#[cfg(test)]
#[path = "footer/tests.rs"]
mod tests;

fn icon_action(
    surface: &crate::ChatUiActionButtonSurface,
    node_id: &str,
    action: ChatUiAction,
    enabled: bool,
) -> Result<UiNode, ChatUiActionError> {
    node::action(UiNodeKind::IconTextButton, &surface.label, node_id, &action).map(|node| {
        node.icon(UiIconProps::new(&surface.icon.svg).role(format!("surface.{node_id}")))
            .variant(UiVariant::Icon)
            .width(UiDimension::px(ACTION_WIDTH))
            .height(UiDimension::px(ACTION_HEIGHT))
            .disabled(!enabled)
    })
}

fn usage(surface: &ChatUiUsageSurface) -> UiNode {
    UiNode::new(UiNodeKind::IconTextButton, "")
        .stable_node_id("chat.composer.usage")
        .accessibility_label(format!(
            "{} / {} ({}%) {} {}",
            surface.used_tokens,
            surface.max_tokens,
            surface.context_percentage,
            surface.context_status,
            surface.account_label
        ))
        .progress(true, surface.context_percentage)
        .variant(UiVariant::Icon)
        .width(UiDimension::px(USAGE_WIDTH))
        .height(UiDimension::px(ACTION_HEIGHT))
}
