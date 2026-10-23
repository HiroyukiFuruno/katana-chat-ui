use crate::kuc::runtime::types::ChatUiRuntimeFrame;
use katana_ui_core::raster_host::UiTreeHostActionHit;
use katana_ui_core::render_model::UiNode;

pub(super) fn action(frame: &ChatUiRuntimeFrame, x: f32, y: f32) -> Option<&UiTreeHostActionHit> {
    frame
        .action_hits
        .iter()
        .rev()
        .find(|hit| hit.contains_point(x, y))
}

pub(super) fn enabled(frame: &ChatUiRuntimeFrame, hit: &UiTreeHostActionHit) -> bool {
    hit.action.enabled
        && node_enabled(frame.tree.root(), hit.action.target.as_str(), false).unwrap_or(false)
}

pub(super) fn permission<'a>(
    frame: &'a ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    prefix: &str,
) -> Option<&'a UiTreeHostActionHit> {
    frame
        .action_hits
        .iter()
        .rev()
        .find(|hit| hit.action.target.as_str().starts_with(prefix) && hit.contains_point(x, y))
}

fn node_enabled(node: &UiNode, target: &str, parent_disabled: bool) -> Option<bool> {
    let disabled = parent_disabled || node.props().disabled;
    if node.id().as_str() == target {
        return Some(!disabled);
    }
    node.children()
        .iter()
        .find_map(|child| node_enabled(child, target, disabled))
}
