mod bounds;
mod hit;
mod position;

use super::super::state::ChatUiRuntime;
use super::super::types::{
    ChatUiRuntimeEffect, ChatUiRuntimeError, ChatUiRuntimeFrame, ChatUiRuntimeInput,
};
use super::composer;
use super::effects;
use crate::ChatUiActionMapper;
use katana_ui_core::text_surface::{TextSurfaceAction, TextSurfaceLayoutAction};

const PROVIDER_SELECTOR: &str = "chat.chrome.provider-selector";
const CONTROL_PREFIX: &str = "chat.composer.control.";
const CONTROL_OPTION: &str = ".option.";
const PERMISSION_PREFIX: &str = "chat.permission.";

pub(super) fn dispatch(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    input: ChatUiRuntimeInput,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) -> Result<Option<ChatUiRuntimeInput>, ChatUiRuntimeError> {
    match input {
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection,
        } => press(runtime, frame, x, y, extend_selection, effects).map(|()| None),
        ChatUiRuntimeInput::PointerDrag { x, y } => {
            drag(runtime, frame, x, y, effects);
            Ok(None)
        }
        ChatUiRuntimeInput::PointerRelease => {
            release(runtime, frame, effects);
            Ok(None)
        }
        input => Ok(Some(input)),
    }
}

pub(super) fn press(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    extend_selection: bool,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) -> Result<(), ChatUiRuntimeError> {
    if route_permission(runtime, frame, x, y, effects)? {
        return Ok(());
    }
    if let Some(hit) = hit::action(frame, x, y) {
        if hit::enabled(frame, hit) {
            return dispatch_action(runtime, hit, effects);
        }
        return Ok(());
    }
    if bounds::inside_composer(frame, x, y) {
        close_dropdown(runtime, effects);
        focus_and_select(runtime, frame, x, y, extend_selection, effects);
        return Ok(());
    }
    toggle_dropdown(runtime, frame, x, y, effects);
    Ok(())
}

fn route_permission(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) -> Result<bool, ChatUiRuntimeError> {
    let Some(hit) = hit::permission(frame, x, y, PERMISSION_PREFIX) else {
        return Ok(runtime.permission_open);
    };
    if hit::enabled(frame, hit) {
        dispatch_action(runtime, hit, effects)?;
    }
    Ok(true)
}

fn dispatch_action(
    runtime: &mut ChatUiRuntime,
    hit: &katana_ui_core::raster_host::UiTreeHostActionHit,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) -> Result<(), ChatUiRuntimeError> {
    effects.push(ChatUiRuntimeEffect::Action(
        ChatUiActionMapper::from_host_action(&hit.action)?,
    ));
    runtime.open_dropdown = None;
    effects.push(ChatUiRuntimeEffect::Repaint);
    Ok(())
}

pub(super) fn drag(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    apply_layout(
        runtime,
        frame,
        TextSurfaceLayoutAction::PointerDrag {
            point: position::point(x, y),
        },
        effects,
    );
}

pub(super) fn release(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    apply_layout(
        runtime,
        frame,
        TextSurfaceLayoutAction::PointerRelease,
        effects,
    );
}

fn focus_and_select(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    extend_selection: bool,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    composer::apply(runtime, TextSurfaceAction::SetFocus(true), effects);
    effects.push(ChatUiRuntimeEffect::FocusComposer);
    apply_layout(
        runtime,
        frame,
        TextSurfaceLayoutAction::PointerPress {
            point: position::point(x, y),
            extend_selection,
        },
        effects,
    );
}

fn apply_layout(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    action: TextSurfaceLayoutAction,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    let outcome = runtime
        .composer
        .apply_layout_action(&frame.composer.layout, action);
    effects::collect(&outcome.events, frame.composer.submit_enabled, effects);
    effects::repaint(outcome.handled, effects);
}

fn toggle_dropdown(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    let Some(node_id) = dropdown_node(frame, x, y) else {
        close_dropdown(runtime, effects);
        return;
    };
    runtime.open_dropdown =
        (runtime.open_dropdown.as_deref() != Some(node_id)).then(|| node_id.to_string());
    effects.push(ChatUiRuntimeEffect::Repaint);
}

fn close_dropdown(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    if runtime.open_dropdown.take().is_some() {
        effects.push(ChatUiRuntimeEffect::Repaint);
    }
}

fn dropdown_node(frame: &ChatUiRuntimeFrame, x: f32, y: f32) -> Option<&str> {
    frame
        .node_hits
        .iter()
        .filter(|hit| hit.contains_point(x, y))
        .map(|hit| hit.node_id.as_str())
        .find(|node_id| is_dropdown(node_id))
}

fn is_dropdown(node_id: &str) -> bool {
    node_id == PROVIDER_SELECTOR
        || (node_id.starts_with(CONTROL_PREFIX) && !node_id.contains(CONTROL_OPTION))
}
