use super::{ChatUiRuntime, ChatUiRuntimeError, ChatUiSurface, surface_width};
use katana_ui_core::raster_host::{UiTreeNodeHit, UiTreeRenderArea};
use katana_ui_core::render_model::UiNode;

pub(super) fn history_layout(
    runtime: &ChatUiRuntime,
    surface: &ChatUiSurface,
    width: u32,
) -> Result<(u32, Vec<u16>), ChatUiRuntimeError> {
    if !surface.history_panel.visible {
        return Ok((0, Vec::new()));
    }
    let content = super::super::super::chrome::history(&surface.history_panel, &[])?;
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

pub(super) fn history_viewport_height(node: &UiNode) -> Option<u32> {
    if node.id().as_str() == "chat.history.scroll" {
        return Some(node.props().scroll_area.viewport_height);
    }
    node.children().iter().find_map(history_viewport_height)
}
