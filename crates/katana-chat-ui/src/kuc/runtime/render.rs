use super::super::content;
use super::super::tree::ChatUiTreeViewport;
use super::composer;
use super::state::ChatUiRuntime;
use super::types::{
    ChatUiRuntimeError, ChatUiRuntimeFrame, ChatUiRuntimeRgba, ChatUiRuntimeViewport,
};
use super::usage;
use crate::{ChatUiSurface, ChatUiTreeBuilder};
use katana_ui_core::raster_host::{Canvas, UiTreeHostActionHit, UiTreeNodeHit, UiTreeRenderArea};
use katana_ui_core::render_model::{UiNode, UiRect, UiTree};

#[path = "render/message_layout.rs"]
mod message_layout;

const COMPOSER_NODE_ID: &str = "chat.composer.input";
struct TreeRaster {
    tree: UiTree,
    canvas: Canvas,
    action_hits: Vec<UiTreeHostActionHit>,
    node_hits: Vec<UiTreeNodeHit>,
}

struct TreeLayout {
    history_content_height: u32,
    history_row_heights: Vec<u16>,
    permission_content_height: u32,
    message_heights: Vec<u16>,
}

struct TreeBuildState<'a> {
    scroll_y: u32,
    message_heights: &'a [u16],
    history_row_heights: &'a [u16],
    hide_thread: bool,
}

pub(super) fn frame(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
) -> Result<ChatUiRuntimeFrame, ChatUiRuntimeError> {
    let mut raster = tree_raster(runtime, surface, viewport)?;
    usage::paint(
        &mut raster.canvas,
        &raster.node_hits,
        surface.usage.context_percentage,
    );
    let bounds = composer_bounds(&mut raster.node_hits, surface, viewport)?;
    let composer = composer::frame(
        runtime,
        &mut raster.canvas,
        bounds,
        viewport,
        &surface.composer.placeholder,
        surface.composer.send_available
            && surface.composer.send_enabled
            && !surface.composer.stop_enabled,
    )?;
    Ok(ChatUiRuntimeFrame {
        rgba: rgba(&raster.canvas),
        tree: raster.tree,
        viewport,
        composer,
        action_hits: raster.action_hits,
        node_hits: raster.node_hits,
    })
}

fn tree_raster(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
) -> Result<TreeRaster, ChatUiRuntimeError> {
    let layout = tree_layout(runtime, surface, viewport)?;
    let tree = build_tree(
        runtime,
        surface,
        viewport,
        &layout,
        TreeBuildState {
            scroll_y: runtime.tree_scroll_y,
            message_heights: &layout.message_heights,
            history_row_heights: &layout.history_row_heights,
            hide_thread: surface.history_panel.visible,
        },
    )?;
    let area = render_area(runtime, viewport);
    let mut canvas = canvas(runtime, viewport);
    runtime
        .renderer
        .render(&mut canvas, tree.tree().root(), area);
    let (action_hits, node_hits) = runtime
        .renderer
        .viewport_interaction_hit_rects(tree.tree().root(), area);
    Ok(TreeRaster {
        tree: tree.tree().clone(),
        canvas,
        action_hits,
        node_hits,
    })
}

fn tree_layout(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
) -> Result<TreeLayout, ChatUiRuntimeError> {
    let (history_content_height, history_row_heights) =
        history_layout(runtime, surface, viewport.width)?;
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
        history_viewport_height(measurement_tree.tree().root())
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

fn build_tree(
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

fn history_layout(
    runtime: &ChatUiRuntime,
    surface: &ChatUiSurface,
    width: u32,
) -> Result<(u32, Vec<u16>), ChatUiRuntimeError> {
    if !surface.history_panel.visible {
        return Ok((0, Vec::new()));
    }
    let content = super::super::chrome::history(&surface.history_panel, &[])?;
    let area = UiTreeRenderArea {
        x: 0,
        y: 0,
        width: usize::from(surface_width(width)),
        height: usize::MAX,
        scroll_y: 0.0,
    };
    let (actions, nodes) = runtime
        .renderer
        .viewport_interaction_hit_rects(&content, area);
    measure_history_rows(actions, nodes, surface.history_panel.sessions.len())
}

fn measure_history_rows(
    actions: Vec<katana_ui_core::raster_host::UiTreeHostActionHit>,
    nodes: Vec<UiTreeNodeHit>,
    row_count: usize,
) -> Result<(u32, Vec<u16>), ChatUiRuntimeError> {
    let mut row_bounds = vec![None; row_count];
    let mut content_bottom = 0_usize;
    for hit in actions {
        content_bottom = content_bottom.max(hit_bottom_usize(hit.rect.y, hit.rect.height));
        if let Some(index) = history_row_index(hit.action.target.as_str()) {
            update_history_row_bounds(&mut row_bounds, index, hit.rect.y, hit.rect.height);
        }
    }
    for hit in nodes {
        content_bottom = content_bottom.max(hit_bottom_usize(hit.rect.y, hit.rect.height));
        if let Some(index) = history_row_index(hit.node_id.as_str()) {
            update_history_row_bounds(&mut row_bounds, index, hit.rect.y, hit.rect.height);
        }
    }
    let row_heights = row_bounds
        .into_iter()
        .map(|bounds| {
            let (top, bottom) = bounds.ok_or(ChatUiRuntimeError::MissingHistoryGeometry)?;
            u16::try_from(bottom.saturating_sub(top))
                .map_err(|_| ChatUiRuntimeError::MissingHistoryGeometry)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let content_height =
        u32::try_from(content_bottom).map_err(|_| ChatUiRuntimeError::MissingHistoryGeometry)?;
    Ok((content_height, row_heights))
}

fn history_row_index(id: &str) -> Option<usize> {
    let suffix = id.strip_prefix("chat.history.session.")?;
    suffix.split('.').next()?.parse().ok()
}

fn update_history_row_bounds(
    bounds: &mut [Option<(usize, usize)>],
    index: usize,
    y: usize,
    height: usize,
) {
    let Some(slot) = bounds.get_mut(index) else {
        return;
    };
    let bottom = hit_bottom_usize(y, height);
    *slot = Some(match *slot {
        Some((top, previous_bottom)) => (top.min(y), previous_bottom.max(bottom)),
        None => (y, bottom),
    });
}

fn hit_bottom_usize(y: usize, height: usize) -> usize {
    y.saturating_add(height)
}

fn history_viewport_height(node: &UiNode) -> Option<u32> {
    if node.id().as_str() == "chat.history.scroll" {
        return Some(node.props().scroll_area.viewport_height);
    }
    node.children().iter().find_map(history_viewport_height)
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

fn hit_bottom(y: usize, height: usize) -> u32 {
    u32::try_from(y)
        .unwrap_or(u32::MAX)
        .saturating_add(u32::try_from(height).unwrap_or(u32::MAX))
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
    let details_width = surface_width(width).saturating_sub(12).max(1);
    let context = super::super::permission::context_node(request, details_width);
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
        .filter(|hit| super::super::permission::is_context_leaf_id(hit.node_id.as_str()))
        .map(|hit| {
            u32::try_from(hit.rect.y)
                .unwrap_or(u32::MAX)
                .saturating_add(u32::try_from(hit.rect.height).unwrap_or(u32::MAX))
        })
        .max()
        .unwrap_or(0)
}

fn surface_width(width: u32) -> u16 {
    u16::try_from(width.min(560)).unwrap_or(560).max(1)
}

fn render_area(_runtime: &ChatUiRuntime, viewport: ChatUiRuntimeViewport) -> UiTreeRenderArea {
    UiTreeRenderArea {
        x: 0,
        y: 0,
        width: viewport.width as usize,
        height: viewport.height as usize,
        scroll_y: 0.0,
    }
}

fn canvas(runtime: &ChatUiRuntime, viewport: ChatUiRuntimeViewport) -> Canvas {
    Canvas::new_scaled(
        viewport.width as usize,
        viewport.height as usize,
        viewport.scale_factor,
        runtime.canvas_background,
    )
}

fn composer_bounds(
    hits: &mut [katana_ui_core::raster_host::UiTreeNodeHit],
    surface: &ChatUiSurface,
    viewport: ChatUiRuntimeViewport,
) -> Result<UiRect, ChatUiRuntimeError> {
    let hit = hits
        .iter_mut()
        .find(|hit| hit.node_id.as_str() == COMPOSER_NODE_ID)
        .ok_or(ChatUiRuntimeError::MissingComposerBounds)?;
    let bounds = super::super::tree::composer_input_bounds(
        viewport.width,
        viewport.height,
        surface.vendor_bar.controls.len(),
    );
    // KUCの汎用TextArea寸法ではなく、上書き描画するTextSurfaceと同じ領域で入力を判定する。
    hit.rect.x = usize::try_from(bounds.x).unwrap_or(0);
    hit.rect.y = usize::try_from(bounds.y).unwrap_or(0);
    hit.rect.width = bounds.width as usize;
    hit.rect.height = bounds.height as usize;
    Ok(bounds)
}

fn rgba(canvas: &Canvas) -> ChatUiRuntimeRgba {
    ChatUiRuntimeRgba {
        width: canvas.width() as u32,
        height: canvas.height() as u32,
        pixels: canvas.pixels().to_vec(),
    }
}
