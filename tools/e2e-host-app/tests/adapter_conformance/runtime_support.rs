use katana_chat_ui::{
    ChatUiAction,
    kuc::{
        ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeFrame, ChatUiRuntimeInput,
        ChatUiRuntimeViewport,
    },
};
use katana_ui_core::render_model::UiNode;

use super::fixture::{STANDARD_HEIGHT, STANDARD_WIDTH, standard_surface};

pub(super) fn runtime() -> Result<ChatUiRuntime, String> {
    ChatUiRuntime::light(&standard_surface()?).map_err(|error| error.to_string())
}

pub(super) fn frame(
    runtime: &mut ChatUiRuntime,
    width: u32,
    height: u32,
) -> Result<ChatUiRuntimeFrame, String> {
    runtime
        .render(
            &standard_surface()?,
            ChatUiRuntimeViewport::new(width, height, 1.0),
        )
        .map_err(|error| error.to_string())
}

pub(super) fn assert_action(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    label: &str,
    expected: ChatUiAction,
) -> Result<(), String> {
    let (x, y) = action_center(frame, label)?;
    let effects = runtime
        .dispatch(
            frame,
            ChatUiRuntimeInput::PointerPress {
                x,
                y,
                extend_selection: false,
            },
        )
        .map_err(|error| error.to_string())?;
    if effects
        .iter()
        .any(|effect| matches!(effect, ChatUiRuntimeEffect::Action(action) if action == &expected))
    {
        return Ok(());
    }
    Err(format!("KUC did not emit typed action {expected:?}"))
}

pub(super) fn select_vendor(
    runtime: &mut ChatUiRuntime,
    current_frame: &ChatUiRuntimeFrame,
) -> Result<(), String> {
    let (x, y) = node_center(current_frame, "chat.chrome.provider-selector")?;
    let _ = runtime
        .dispatch(
            current_frame,
            ChatUiRuntimeInput::PointerPress {
                x,
                y,
                extend_selection: false,
            },
        )
        .map_err(|error| error.to_string())?;
    let open = frame(runtime, STANDARD_WIDTH, STANDARD_HEIGHT)?;
    assert_action(
        runtime,
        &open,
        "codex-cli",
        ChatUiAction::SelectVendor {
            vendor_id: "codex-cli".to_string(),
        },
    )
}

pub(super) fn assert_header_action(
    frame: &ChatUiRuntimeFrame,
    action: &str,
    composer_y: usize,
) -> Result<(), String> {
    let (_, y) = action_center(frame, action)?;
    if y >= composer_y as f32 {
        return Err(format!("KUC action {action} was not in the header"));
    }
    Ok(())
}

pub(super) fn composer_rect(frame: &ChatUiRuntimeFrame) -> (usize, usize, usize) {
    (
        frame.composer.bounds.x as usize,
        frame.composer.bounds.y as usize,
        frame.composer.bounds.width as usize,
    )
}

pub(super) fn node_rect(
    frame: &ChatUiRuntimeFrame,
    id: &str,
) -> Result<(usize, usize, usize), String> {
    frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == id)
        .map(|hit| (hit.rect.x, hit.rect.y, hit.rect.width))
        .ok_or_else(|| format!("KUC node is missing: {id}"))
}

pub(super) fn node_center(frame: &ChatUiRuntimeFrame, id: &str) -> Result<(f32, f32), String> {
    frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == id)
        .map(|hit| hit.rect.center_point())
        .ok_or_else(|| format!("KUC node is missing: {id}"))
}

pub(super) fn action_center(frame: &ChatUiRuntimeFrame, value: &str) -> Result<(f32, f32), String> {
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(value))
        .map(|hit| hit.center_point())
        .ok_or_else(|| format!("KUC action hit is missing: {value}"))
}

pub(super) fn action_left(frame: &ChatUiRuntimeFrame, value: &str) -> Result<usize, String> {
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(value))
        .map(|hit| hit.rect.x)
        .ok_or_else(|| format!("KUC action hit is missing: {value}"))
}

pub(super) fn node<'a>(frame: &'a ChatUiRuntimeFrame, id: &str) -> Result<&'a UiNode, String> {
    find_node(frame.tree.root(), id).ok_or_else(|| format!("KUC tree node is missing: {id}"))
}

fn find_node<'a>(node: &'a UiNode, id: &str) -> Option<&'a UiNode> {
    if node.id().as_str() == id {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_node(child, id))
}

pub(super) fn has_node_fragment(frame: &ChatUiRuntimeFrame, value: &str) -> bool {
    frame
        .node_hits
        .iter()
        .any(|hit| hit.node_id.as_str().contains(value))
}
