use super::{ChatUiAction, ChatUiActionError, node};
use crate::{ChatUiPermissionRequestSurface, ChatUiVendorBarSurface};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiDisplay, UiLayoutAxis, UiNode, UiNodeKind, UiOverflow,
    UiScrollAreaProps,
};

pub(super) fn panel(
    surface: &ChatUiVendorBarSurface,
    scroll_y: u32,
    panel_width: u16,
    panel_height: u16,
    detail_viewport_height: u16,
    content_height: u32,
) -> Result<UiNode, ChatUiActionError> {
    let Some(request) = &surface.permission_request else {
        return Ok(permission_panel_node(panel_height).visible(false));
    };
    let panel = permission_panel_node(panel_height);
    let details_width = panel_width.saturating_sub(12).max(1);
    let details = node::container(
        UiNodeKind::ScrollArea,
        "承認対象の操作内容",
        "chat.permission.details-scroll",
    )
    .common(
        UiCommonProps::default()
            .width(UiDimension::px(details_width))
            .height(UiDimension::px(detail_viewport_height))
            .display(UiDisplay::Flex)
            .layout_axis(UiLayoutAxis::Vertical)
            .overflow(UiOverflow::Auto),
    );
    let details = details
        .child(context_node(request, details_width))
        .scroll_area(UiScrollAreaProps {
            viewport_width: u32::from(details_width),
            viewport_height: u32::from(detail_viewport_height),
            content_width: u32::from(details_width),
            content_height: content_height.max(u32::from(detail_viewport_height)),
            offset_y: scroll_y
                .min(content_height.saturating_sub(u32::from(detail_viewport_height))),
            ..UiScrollAreaProps::default()
        });
    append_permission_actions(panel.child(details), request)
}

fn permission_panel_node(panel_height: u16) -> UiNode {
    node::container(UiNodeKind::Panel, "実行の確認", "chat.permission").common(
        UiCommonProps::default()
            .display(UiDisplay::Flex)
            .layout_axis(UiLayoutAxis::Vertical)
            .height(UiDimension::px(panel_height))
            .overflow(UiOverflow::Hidden),
    )
}

fn append_permission_actions(
    mut panel: UiNode,
    request: &ChatUiPermissionRequestSurface,
) -> Result<UiNode, ChatUiActionError> {
    for (index, option) in request.options.iter().enumerate() {
        panel = panel.child(node::action(
            UiNodeKind::Button,
            &option.label,
            &format!("chat.permission.option.{index}"),
            &ChatUiAction::RespondPermission {
                request_id: request.request_id,
                option_id: Some(option.id.clone()),
            },
        )?);
    }
    Ok(panel.child(node::action(
        UiNodeKind::Button,
        "キャンセル",
        "chat.permission.cancel",
        &ChatUiAction::RespondPermission {
            request_id: request.request_id,
            option_id: None,
        },
    )?))
}

pub(super) fn context_node(request: &ChatUiPermissionRequestSurface, width: u16) -> UiNode {
    let mut context = node::container(UiNodeKind::Stack, "", "chat.permission.context").common(
        UiCommonProps::default()
            .width(UiDimension::px(width))
            .display(UiDisplay::Flex)
            .layout_axis(UiLayoutAxis::Vertical),
    );
    for (id, role, text) in context_lines(request) {
        context = context.child(node::text(&text, &id, &role));
    }
    context
}

pub(super) fn is_context_leaf_id(node_id: &str) -> bool {
    node_id == "chat.permission.tool-call-id"
        || node_id == "chat.permission.title"
        || node_id == "chat.permission.details-unavailable"
        || node_id.starts_with("chat.permission.context.")
}

fn context_lines(request: &ChatUiPermissionRequestSurface) -> Vec<(String, String, String)> {
    let mut lines = Vec::new();
    append_context_identity(&mut lines, request);
    append_context_details(&mut lines, request);
    lines
}

fn append_context_identity(
    lines: &mut Vec<(String, String, String)>,
    request: &ChatUiPermissionRequestSurface,
) {
    if let Some(tool_call_id) = &request.tool_call_id {
        lines.push((
            "chat.permission.tool-call-id".to_string(),
            "body".to_string(),
            format!("操作ID: {tool_call_id}"),
        ));
    }
    if let Some(title) = &request.title {
        lines.push((
            "chat.permission.title".to_string(),
            "heading".to_string(),
            title.clone(),
        ));
    }
}

fn append_context_details(
    lines: &mut Vec<(String, String, String)>,
    request: &ChatUiPermissionRequestSurface,
) {
    if request.details.is_empty() {
        lines.push((
            "chat.permission.details-unavailable".to_string(),
            "body".to_string(),
            "操作の詳細は提供されていません。".to_string(),
        ));
    } else {
        lines.extend(request.details.iter().enumerate().map(|(index, detail)| {
            (
                format!("chat.permission.context.detail.{index}"),
                "body".to_string(),
                detail.clone(),
            )
        }));
    }
}
