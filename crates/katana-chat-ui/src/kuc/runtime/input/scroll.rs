use super::super::state::ChatUiRuntime;
use super::super::types::{ChatUiRuntimeEffect, ChatUiRuntimeFrame, ChatUiRuntimeInput};
use super::composer;
use katana_ui_core::raster_host::UiTreeRenderArea;
use katana_ui_core::render_model::{UiDimension, UiNode, UiRect};
use katana_ui_core::text_surface::TextSurfaceAction;

pub(super) fn dispatch(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    input: ChatUiRuntimeInput,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) -> Option<ChatUiRuntimeInput> {
    let ChatUiRuntimeInput::Scroll {
        x,
        y,
        delta_x,
        delta_y,
    } = input
    else {
        return Some(input);
    };
    apply(runtime, frame, x, y, delta_x, delta_y, effects);
    None
}

pub(super) fn apply(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    delta_x: i32,
    delta_y: i32,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    if runtime.permission_open {
        if inside_permission_details(runtime, frame, x, y) {
            let maximum = permission_scroll_max(frame.tree.root());
            runtime.permission_scroll_y =
                scroll_offset(runtime.permission_scroll_y, delta_y).min(maximum);
            effects.push(ChatUiRuntimeEffect::Repaint);
        }
        return;
    }
    if inside_history(frame, x, y) {
        runtime.history_scroll_y = scroll_offset(runtime.history_scroll_y, delta_y)
            .min(history_scroll_max(frame.tree.root()));
        effects.push(ChatUiRuntimeEffect::Repaint);
        return;
    }
    if inside_composer(frame, x, y) {
        composer::apply(
            runtime,
            TextSurfaceAction::ScrollBy { delta_x, delta_y },
            effects,
        );
        return;
    }
    runtime.tree_scroll_y = scroll_offset(runtime.tree_scroll_y, delta_y);
    effects.push(ChatUiRuntimeEffect::Repaint);
}

fn inside_history(frame: &ChatUiRuntimeFrame, x: f32, y: f32) -> bool {
    history_scroll_rect(frame.tree.root()).is_some_and(|rect| {
        x >= rect.x as f32
            && y >= rect.y as f32
            && x < rect.x.saturating_add_unsigned(rect.width) as f32
            && y < rect.y.saturating_add_unsigned(rect.height) as f32
    })
}

fn history_scroll_rect(node: &UiNode) -> Option<UiRect> {
    if node.id().as_str() == "chat.history.scroll" {
        let margin = &node.props().common.margin;
        let (UiDimension::Px(x), UiDimension::Px(y)) = (&margin.left, &margin.top) else {
            return None;
        };
        let scroll = &node.props().scroll_area;
        return Some(UiRect::new(
            i32::from(*x),
            i32::from(*y),
            scroll.viewport_width,
            scroll.viewport_height,
        ));
    }
    node.children().iter().find_map(history_scroll_rect)
}

fn history_scroll_max(node: &UiNode) -> u32 {
    if node.id().as_str() == "chat.history.scroll" {
        let scroll = &node.props().scroll_area;
        return scroll.content_height.saturating_sub(scroll.viewport_height);
    }
    node.children()
        .iter()
        .map(history_scroll_max)
        .find(|maximum| *maximum > 0)
        .unwrap_or(0)
}

fn inside_permission_details(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
) -> bool {
    let hits = permission_document_hits(runtime, frame);
    let Some((panel_top, options_top)) = permission_context_bounds(frame) else {
        return false;
    };
    hits.iter().any(|hit| {
        super::super::super::permission::is_context_leaf_id(hit.node_id.as_str())
            && point_in_context(hit, x, y, panel_top, options_top)
    })
}

fn permission_document_hits(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Vec<katana_ui_core::raster_host::UiTreeNodeHit> {
    runtime.renderer.document_node_hit_rects(
        frame.tree.root(),
        UiTreeRenderArea {
            x: 0,
            y: 0,
            width: frame.viewport.width as usize,
            height: frame.viewport.height as usize,
            scroll_y: 0.0,
        },
    )
}

fn permission_context_bounds(frame: &ChatUiRuntimeFrame) -> Option<(usize, usize)> {
    let panel = frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "chat.permission")
        .map(|hit| hit.rect);
    let first_option_y = frame
        .action_hits
        .iter()
        .filter(|hit| {
            hit.action
                .target
                .as_str()
                .starts_with("chat.permission.option.")
        })
        .map(|hit| hit.rect.y)
        .min();
    Some((panel?.y, first_option_y?))
}

fn point_in_context(
    hit: &katana_ui_core::raster_host::UiTreeNodeHit,
    x: f32,
    y: f32,
    panel_top: usize,
    options_top: usize,
) -> bool {
    y >= panel_top as f32
        && y < options_top as f32
        && hit.rect.y >= panel_top
        && hit.rect.y < options_top
        && x >= hit.rect.x as f32
        && y >= hit.rect.y as f32
        && x < hit.rect.x.saturating_add(hit.rect.width) as f32
        && y < hit.rect.y.saturating_add(hit.rect.height) as f32
}

fn permission_scroll_max(node: &katana_ui_core::render_model::UiNode) -> u32 {
    if node.id().as_str() == "chat.permission.details-scroll" {
        return node
            .props()
            .scroll_area
            .content_height
            .saturating_sub(node.props().scroll_area.viewport_height);
    }
    node.children()
        .iter()
        .map(permission_scroll_max)
        .find(|maximum| *maximum > 0)
        .unwrap_or(0)
}

fn scroll_offset(current: u32, delta: i32) -> u32 {
    if delta.is_negative() {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta as u32)
    }
}

fn inside_composer(frame: &ChatUiRuntimeFrame, x: f32, y: f32) -> bool {
    let bounds = frame.composer.bounds;
    x >= bounds.x as f32
        && y >= bounds.y as f32
        && x < bounds.x.saturating_add_unsigned(bounds.width) as f32
        && y < bounds.y.saturating_add_unsigned(bounds.height) as f32
}
