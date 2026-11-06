use super::{
    ChatUiRuntime, ChatUiRuntimeError, ChatUiRuntimeViewport, ChatUiSurface, ChatUiTreeBuilder,
    ChatUiTreeViewport, TreeBuildState, TreeLayout, content, history, message_layout,
    surface_width,
};
use katana_ui_core::raster_host::UiTreeRenderArea;
use katana_ui_core::render_model::UiNode;

pub(super) fn tree_layout(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
) -> Result<TreeLayout, ChatUiRuntimeError> {
    let (history_content_height, history_row_heights) =
        history::history_layout(runtime, surface, viewport.width)?;
    let mut layout = TreeLayout {
        history_content_height,
        history_row_heights,
        permission_content_height: permission_content_height(runtime, surface, viewport.width),
        message_heights: Vec::new(),
    };
    let measurement_tree = build_tree(
        runtime,
        surface,
        viewport,
        &layout,
        TreeBuildState {
            scroll_y: 0,
            message_heights: &[],
            history_row_heights: &[],
            hide_thread: false,
        },
    )?;
    finalize_tree_layout(runtime, surface, viewport, &mut layout, &measurement_tree)?;
    Ok(layout)
}

fn finalize_tree_layout(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
    layout: &mut TreeLayout,
    measurement_tree: &crate::ChatUiTree,
) -> Result<(), ChatUiRuntimeError> {
    let history_height = if surface.history_panel.visible {
        history::history_viewport_height(measurement_tree.tree().root())
            .ok_or(ChatUiRuntimeError::MissingHistoryGeometry)?
    } else {
        0
    };
    runtime.sync_history_scroll(
        surface.history_panel.visible,
        &surface.history_panel.sessions,
        layout.history_content_height,
        history_height,
    );
    layout.message_heights = measure_messages(runtime, surface, viewport, measurement_tree)?;
    clamp_thread_scroll(
        runtime,
        measurement_tree.tree().root(),
        &layout.message_heights,
    )?;
    Ok(())
}

fn measure_messages(
    runtime: &ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
    measurement_tree: &crate::ChatUiTree,
) -> Result<Vec<u16>, ChatUiRuntimeError> {
    let minimum_heights = content::minimum_message_heights(
        &surface.message_list.messages,
        surface_width(viewport.width),
    );
    measured_message_heights(
        runtime,
        surface,
        viewport,
        measurement_tree,
        &minimum_heights,
    )
}

fn clamp_thread_scroll(
    runtime: &mut ChatUiRuntime,
    root: &katana_ui_core::render_model::UiNode,
    message_heights: &[u16],
) -> Result<(), ChatUiRuntimeError> {
    let viewport_height =
        thread_viewport_height(root).ok_or(ChatUiRuntimeError::MissingMessageGeometry {
            message_id: "thread".to_string(),
            component: "scroll viewport".to_string(),
        })?;
    let content_height = content::thread_content_height(message_heights).max(viewport_height);
    runtime.tree_scroll_y = runtime
        .tree_scroll_y
        .min(content_height.saturating_sub(viewport_height));
    Ok(())
}

pub(super) fn build_tree(
    runtime: &ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
    layout: &TreeLayout,
    state: TreeBuildState<'_>,
) -> Result<crate::ChatUiTree, ChatUiRuntimeError> {
    ChatUiTreeBuilder::build_for_viewport(
        surface,
        runtime.open_dropdown.as_deref(),
        ChatUiTreeViewport {
            scroll_y: state.scroll_y,
            hide_thread: state.hide_thread,
            history_scroll_y: runtime.history_scroll_y,
            history_content_height: layout.history_content_height,
            history_row_heights: state.history_row_heights,
            permission_scroll_y: runtime.permission_scroll_y,
            permission_content_height: layout.permission_content_height,
            message_heights: state.message_heights,
            width: viewport.width,
            height: viewport.height,
        },
    )
    .map_err(Into::into)
}

fn thread_viewport_height(node: &UiNode) -> Option<u32> {
    if node.id().as_str() == "chat.thread" {
        return Some(node.props().scroll_area.viewport_height);
    }
    node.children().iter().find_map(thread_viewport_height)
}

fn measured_message_heights(
    runtime: &ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
    tree: &crate::ChatUiTree,
    minimum_heights: &[u16],
) -> Result<Vec<u16>, ChatUiRuntimeError> {
    let measured_hits = runtime
        .renderer
        .document_node_hit_rects(tree.tree().root(), document_area(viewport));
    message_layout::measure(tree.tree().root(), surface, minimum_heights, &measured_hits)
}

fn document_area(viewport: ChatUiRuntimeViewport) -> UiTreeRenderArea {
    UiTreeRenderArea {
        x: 0,
        y: 0,
        width: viewport.width as usize,
        height: usize::MAX,
        scroll_y: 0.0,
    }
}

fn permission_content_height(runtime: &ChatUiRuntime, surface: &ChatUiSurface, width: u32) -> u32 {
    let Some(request) = &surface.vendor_bar.permission_request else {
        return 0;
    };
    let details_width = surface_width(width)
        .saturating_sub(super::super::super::permission::PANEL_HORIZONTAL_INSET)
        .max(1);
    let context = super::super::super::permission::context_node(request, details_width);
    let measured = runtime.renderer.document_node_hit_rects(
        &context,
        UiTreeRenderArea {
            x: 0,
            y: 0,
            width: usize::from(details_width),
            height: usize::MAX,
            scroll_y: 0.0,
        },
    );
    measured
        .iter()
        .filter(|hit| super::super::super::permission::is_context_leaf_id(hit.node_id.as_str()))
        .map(|hit| {
            u32::try_from(hit.rect.y)
                .unwrap_or(u32::MAX)
                .saturating_add(u32::try_from(hit.rect.height).unwrap_or(u32::MAX))
        })
        .max()
        .unwrap_or(0)
}
