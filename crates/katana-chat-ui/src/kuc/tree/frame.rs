use super::{
    ChatUiActionError, ChatUiTree, ChatUiTreeViewport, chrome, composer, content, node,
    output_handoff,
};
use crate::ChatUiSurface;
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiEdgeInsets, UiNode, UiNodeKind, UiPosition, UiScrollAreaProps,
    UiTree,
};

mod dropdown;

const HEADER_HEIGHT: u16 = 40;
const MAX_BODY_WIDTH: u16 = 560;

pub(crate) fn composer_input_bounds(
    width: u32,
    height: u32,
    control_count: usize,
) -> katana_ui_core::render_model::UiRect {
    let layout = ChatFrameLayout::from_viewport(width, height, control_count);
    katana_ui_core::render_model::UiRect::new(
        i32::from(layout.side_margin),
        i32::from(layout.composer_top()),
        u32::from(layout.body_width),
        u32::from(composer::INPUT_HEIGHT),
    )
}

pub(super) fn build(
    surface: &ChatUiSurface,
    open_dropdown: Option<&str>,
    viewport: ChatUiTreeViewport<'_>,
) -> Result<ChatUiTree, ChatUiActionError> {
    let layout = ChatFrameLayout::from_viewport(
        viewport.width,
        viewport.height,
        surface.vendor_bar.controls.len(),
    );
    let root = frame_root(surface, open_dropdown, viewport, layout)?;
    Ok(ChatUiTree {
        source: surface.clone(),
        tree: UiTree::new(root),
    })
}

fn frame_root(
    surface: &ChatUiSurface,
    open_dropdown: Option<&str>,
    viewport: ChatUiTreeViewport<'_>,
    layout: ChatFrameLayout,
) -> Result<UiNode, ChatUiActionError> {
    let mut root = node::container(UiNodeKind::Stack, "", "chat.root")
        .common(root_common(layout))
        .child(header(surface, open_dropdown, layout)?);
    if !viewport.hide_thread {
        root = root.child(thread(
            surface,
            layout,
            viewport.scroll_y,
            viewport.message_heights,
        )?);
    }
    root = root.child(composer(surface, open_dropdown, layout)?);
    root = append_history_overlay(root, surface, &viewport, layout)?;
    root = dropdown::append(root, surface, open_dropdown, layout)?;
    append_permission_overlay(root, surface, viewport, layout)
}

fn append_permission_overlay(
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
        super::super::permission::panel(
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

fn append_history_overlay(
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
    let action_count = surface
        .vendor_bar
        .permission_request
        .as_ref()
        .map_or(1, |request| request.options.len().saturating_add(1));
    let action_height = u16::try_from(action_count)
        .unwrap_or(u16::MAX)
        .saturating_mul(32);
    let available = layout.viewport_height.saturating_sub(HEADER_HEIGHT);
    let panel_height = available.min(action_height.saturating_add(124));
    let details_height = panel_height.saturating_sub(action_height.saturating_add(24));
    (panel_height, details_height)
}

fn header(
    surface: &ChatUiSurface,
    open_dropdown: Option<&str>,
    layout: ChatFrameLayout,
) -> Result<UiNode, ChatUiActionError> {
    chrome::chrome(
        &surface.chrome,
        &surface.vendor_bar,
        open_dropdown,
        layout.body_width,
    )
    .map(|node| positioned(node, "chat.frame.header", layout, 0, HEADER_HEIGHT))
}

fn thread(
    surface: &ChatUiSurface,
    layout: ChatFrameLayout,
    scroll_y: u32,
    message_heights: &[u16],
) -> Result<UiNode, ChatUiActionError> {
    let row_heights = content::effective_message_heights(
        &surface.message_list.messages,
        layout.body_width,
        message_heights,
    );
    let content_height =
        content::thread_content_height(&row_heights).max(u32::from(layout.thread_height));
    let mut content = content::thread_for_width(
        &surface.message_list.messages,
        layout.body_width,
        &row_heights,
    )?
    .scroll_area(thread_scroll_area(layout, content_height, scroll_y));
    if !surface.output_handoff.outputs.is_empty() {
        content = content.child(output_handoff(surface)?);
    }
    Ok(positioned(
        content,
        "chat.frame.thread",
        layout,
        HEADER_HEIGHT,
        layout.thread_height,
    ))
}

fn thread_scroll_area(
    layout: ChatFrameLayout,
    content_height: u32,
    scroll_y: u32,
) -> UiScrollAreaProps {
    let viewport_height = u32::from(layout.thread_height);
    UiScrollAreaProps {
        viewport_width: u32::from(layout.body_width),
        viewport_height,
        content_width: u32::from(layout.body_width),
        content_height,
        offset_y: scroll_y.min(content_height.saturating_sub(viewport_height)),
        ..UiScrollAreaProps::default()
    }
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

fn composer(
    surface: &ChatUiSurface,
    open_dropdown: Option<&str>,
    layout: ChatFrameLayout,
) -> Result<UiNode, ChatUiActionError> {
    composer::composer(
        &surface.composer,
        &surface.vendor_bar,
        &surface.usage,
        open_dropdown,
        layout.body_width,
    )
    .map(|node| {
        positioned(
            node,
            "chat.frame.composer",
            layout,
            layout.composer_top(),
            layout.composer_height,
        )
    })
}

fn root_common(layout: ChatFrameLayout) -> UiCommonProps {
    UiCommonProps::default()
        .width(UiDimension::px(layout.viewport_width))
        .height(UiDimension::px(layout.viewport_height))
}

fn positioned(
    node: UiNode,
    node_id: &str,
    layout: ChatFrameLayout,
    top: u16,
    height: u16,
) -> UiNode {
    node::container(UiNodeKind::Stack, "", node_id)
        .common(
            UiCommonProps::default()
                .width(UiDimension::px(layout.body_width))
                .height(UiDimension::px(height))
                .position(UiPosition::Absolute)
                .margin(UiEdgeInsets {
                    top: UiDimension::px(top),
                    right: UiDimension::px(0),
                    bottom: UiDimension::px(0),
                    left: UiDimension::px(layout.side_margin),
                }),
        )
        .child(node)
}

#[derive(Clone, Copy)]
pub(super) struct ChatFrameLayout {
    viewport_width: u16,
    viewport_height: u16,
    body_width: u16,
    side_margin: u16,
    thread_height: u16,
    composer_height: u16,
}

impl ChatFrameLayout {
    fn from_viewport(width: u32, height: u32, control_count: usize) -> Self {
        let viewport_width = clamp_dimension(width).max(1);
        let body_width = viewport_width.min(MAX_BODY_WIDTH);
        let composer_height = composer::height(control_count, body_width);
        let minimum_height = HEADER_HEIGHT
            .saturating_add(composer_height)
            .saturating_add(1);
        let viewport_height = clamp_dimension(height).max(minimum_height);
        let side_margin = viewport_width.saturating_sub(body_width) / 2;
        let thread_height = viewport_height.saturating_sub(HEADER_HEIGHT + composer_height);
        Self {
            viewport_width,
            viewport_height,
            body_width,
            side_margin,
            thread_height,
            composer_height,
        }
    }

    fn composer_top(self) -> u16 {
        HEADER_HEIGHT.saturating_add(self.thread_height)
    }
}

fn clamp_dimension(value: u32) -> u16 {
    u16::try_from(value.min(u32::from(u16::MAX))).unwrap_or(u16::MAX)
}
