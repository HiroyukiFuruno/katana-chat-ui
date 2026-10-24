use super::{ChatUiAction, ChatUiActionError, ChatUiActionMapper};
use katana_ui_core::render_model::{UiNode, UiNodeKind, UiTextProps, UiTextSpan, UiTextWrapMode};

pub(super) fn container(kind: UiNodeKind, label: &str, node_id: &str) -> UiNode {
    UiNode::new(kind, label).stable_node_id(node_id)
}

pub(super) fn text(label: &str, node_id: &str, role: &str) -> UiNode {
    UiNode::new(UiNodeKind::Text, label)
        .stable_node_id(node_id)
        .text(UiTextProps {
            role: role.to_string(),
            wrap: UiTextWrapMode::Wrap,
            spans: vec![UiTextSpan::plain(label)],
            ..UiTextProps::default()
        })
}

pub(super) fn action(
    kind: UiNodeKind,
    label: &str,
    node_id: &str,
    value: &ChatUiAction,
) -> Result<UiNode, ChatUiActionError> {
    let action = ChatUiActionMapper::host_action(value, label)?;
    Ok(container(kind, label, node_id).host_action(action))
}

pub(super) fn action_with_visual_label(
    kind: UiNodeKind,
    action_label: &str,
    visual_label: &str,
    node_id: &str,
    value: &ChatUiAction,
) -> Result<UiNode, ChatUiActionError> {
    let action = ChatUiActionMapper::host_action(value, action_label)?;
    Ok(container(kind, visual_label, node_id).host_action(action))
}
