use super::{
    ChatFrameLayout, ChatUiActionError, ChatUiSurface, ChatUiTreeViewport, HEADER_HEIGHT, chrome,
    node, positioned,
};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiEdgeInsets, UiNode, UiNodeKind, UiPosition, UiScrollAreaProps,
};

pub(super) fn append_permission_overlay(
    root: UiNode,
    surface: &ChatUiSurface,
    viewport: ChatUiTreeViewport<'_>,
    layout: ChatFrameLayout,
) -> Result<UiNode, ChatUiActionError> {
    let Some(_) = surface.vendor_bar.permission_request else {
        return Ok(root);
    };
    let (panel_height, detail_viewport_height) = permission_heights(surface, layout);
    Ok(root.child(positioned(
        super::super::super::permission::panel(
            &surface.vendor_bar,
            viewport.permission_scroll_y,
            layout.body_width,
            panel_height,
            detail_viewport_height,
            viewport.permission_content_height,
        )?,
        "chat.frame.permission",
        layout,
        HEADER_HEIGHT,
        panel_height,
    )))
}

pub(super) fn append_history_overlay(
    root: UiNode,
    surface: &ChatUiSurface,
    viewport: &ChatUiTreeViewport<'_>,
    layout: ChatFrameLayout,
) -> Result<UiNode, ChatUiActionError> {
    if !surface.history_panel.visible {
        return Ok(root);
    }
    Ok(root.child(history_overlay(
        surface,
        layout,
        viewport.history_scroll_y,
        viewport.history_content_height,
        viewport.history_row_heights,
    )?))
}

fn permission_heights(surface: &ChatUiSurface, layout: ChatFrameLayout) -> (u16, u16) {
    const ACTION_ROW_HEIGHT: u16 = 32;
    const DETAIL_TARGET_HEIGHT: u16 = 124;
    const PANEL_VERTICAL_PADDING: u16 = 24;
    let action_count = surface
        .vendor_bar
        .permission_request
        .as_ref()
        .map_or(1, |request| request.options.len().saturating_add(1));
    let action_height = u16::try_from(action_count)
        .unwrap_or(u16::MAX)
        .saturating_mul(ACTION_ROW_HEIGHT);
    let available = layout.viewport_height.saturating_sub(HEADER_HEIGHT);
    let panel_height = available.min(action_height.saturating_add(DETAIL_TARGET_HEIGHT));
    let details_height =
        panel_height.saturating_sub(action_height.saturating_add(PANEL_VERTICAL_PADDING));
    (panel_height, details_height)
}

fn history_overlay(
    surface: &ChatUiSurface,
    layout: ChatFrameLayout,
    scroll_y: u32,
    content_height: u32,
    row_heights: &[u16],
) -> Result<UiNode, ChatUiActionError> {
    let viewport_height = u32::from(layout.thread_height);
    let content_height = content_height.max(viewport_height);
    Ok(
        node::container(UiNodeKind::ScrollArea, "", "chat.history.scroll")
            .common(
                UiCommonProps::default()
                    .width(UiDimension::px(layout.body_width))
                    .height(UiDimension::px(layout.thread_height))
                    .position(UiPosition::Absolute)
                    .margin(UiEdgeInsets {
                        top: UiDimension::px(HEADER_HEIGHT),
                        right: UiDimension::px(0),
                        bottom: UiDimension::px(0),
                        left: UiDimension::px(layout.side_margin),
                    }),
            )
            .child(chrome::history(&surface.history_panel, row_heights)?)
            .scroll_area(UiScrollAreaProps {
                viewport_width: u32::from(layout.body_width),
                viewport_height,
                content_width: u32::from(layout.body_width),
                content_height,
                offset_y: scroll_y.min(content_height.saturating_sub(viewport_height)),
                ..UiScrollAreaProps::default()
            }),
    )
}
