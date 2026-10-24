use super::{ChatUiAction, ChatUiActionError, node};
use crate::{
    ChatUiActionButtonSurface, ChatUiChromeSurface, ChatUiHistoryPanelSurface,
    ChatUiVendorBarSurface,
};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiDisplay, UiIconProps, UiLayoutAxis, UiNode, UiNodeKind, UiVariant,
};

pub(super) fn chrome(
    surface: &ChatUiChromeSurface,
    vendor: &ChatUiVendorBarSurface,
    open_dropdown: Option<&str>,
    available_width: u16,
) -> Result<UiNode, ChatUiActionError> {
    Ok(node::container(UiNodeKind::Row, "", "chat.chrome")
        .common(
            UiCommonProps::default()
                .display(UiDisplay::Flex)
                .layout_axis(UiLayoutAxis::Horizontal)
                .gap(UiDimension::px(8)),
        )
        .child(super::vendor::selector(vendor, open_dropdown)?)
        .child(title(surface, available_width))
        .child(header_action(
            &surface.new_chat,
            "chat.chrome.new-chat",
            ChatUiAction::NewChat,
        )?)
        .child(header_action(
            &surface.history,
            "chat.chrome.history",
            ChatUiAction::ToggleHistory,
        )?))
}

fn title(surface: &ChatUiChromeSurface, available_width: u16) -> UiNode {
    let reserved = super::vendor::CONTROL_WIDTH + 2 * 32 + 3 * 8;
    let width = available_width.saturating_sub(reserved).min(240);
    node::text(&surface.title, "chat.chrome.title", "chat-title")
        .common(UiCommonProps::default().width(UiDimension::px(width)))
}

fn header_action(
    surface: &ChatUiActionButtonSurface,
    node_id: &str,
    action: ChatUiAction,
) -> Result<UiNode, ChatUiActionError> {
    node::action(UiNodeKind::IconTextButton, &surface.label, node_id, &action).map(|node| {
        node.icon(UiIconProps::new(&surface.icon.svg).role(format!("surface.{node_id}")))
            .variant(UiVariant::Icon)
            .width(UiDimension::px(32))
            .height(UiDimension::px(28))
            .disabled(!surface.enabled)
    })
}

pub(super) fn history(
    surface: &ChatUiHistoryPanelSurface,
    row_heights: &[u16],
) -> Result<UiNode, ChatUiActionError> {
    let mut panel =
        node::container(UiNodeKind::Panel, &surface.label, "chat.history").visible(surface.visible);
    if surface.sessions.is_empty() {
        panel = panel.child(node::text(
            &surface.empty_label,
            "chat.history.empty-label",
            "history-empty-label",
        ));
    }
    for (index, session) in surface.sessions.iter().enumerate() {
        panel = panel.child(history_row(
            index,
            session,
            row_heights.get(index).copied(),
        )?);
    }
    Ok(panel)
}

fn history_row(
    index: usize,
    session: &crate::ChatUiHistorySessionSurface,
    measured_height: Option<u16>,
) -> Result<UiNode, ChatUiActionError> {
    let action = ChatUiAction::SelectHistory {
        session_id: session.session_id.clone(),
    };
    let accessibility_label = format!(
        "{} {} {} {} {}",
        session.title,
        session.provider_id,
        session.provider_label,
        session.updated_at_label,
        session.preview
    );
    let mut row = node::action_with_visual_label(
        UiNodeKind::SelectionList,
        &session.title,
        "",
        &format!("chat.history.session.{index}"),
        &action,
    )?
    .accessibility_label(accessibility_label);
    if let Some(height) = measured_height {
        row = row.height(UiDimension::px(height));
    }
    for child in history_row_children(index, session) {
        row = row.child(child);
    }
    Ok(row)
}

fn history_row_children(index: usize, session: &crate::ChatUiHistorySessionSurface) -> [UiNode; 3] {
    [
        node::text(
            &session.title,
            &format!("chat.history.session.{index}.title"),
            "history-session-title",
        ),
        node::text(
            &format!("{} · {}", session.provider_label, session.updated_at_label),
            &format!("chat.history.session.{index}.meta"),
            "history-session-meta",
        ),
        node::text(
            &session.preview,
            &format!("chat.history.session.{index}.preview"),
            "history-session-preview",
        ),
    ]
}
