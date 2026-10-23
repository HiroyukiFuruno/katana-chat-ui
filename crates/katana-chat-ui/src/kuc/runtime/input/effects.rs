use super::super::state::ChatUiRuntime;
use super::super::types::{ChatUiRuntimeClipboard, ChatUiRuntimeEffect};
use crate::ChatUiAction;
use katana_ui_core::atom::TextAreaEvent;
use katana_ui_core::text_surface::TextSurfaceEvent;

pub(super) fn collect(
    events: &[TextSurfaceEvent],
    submit_enabled: bool,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    for event in events {
        match event {
            TextSurfaceEvent::TextArea(TextAreaEvent::Change(value)) => {
                effects.push(ChatUiRuntimeEffect::DraftChanged(value.clone()));
            }
            TextSurfaceEvent::TextArea(TextAreaEvent::Submit(_)) if submit_enabled => {
                effects.push(ChatUiRuntimeEffect::Action(ChatUiAction::Submit));
            }
            _ => {}
        }
    }
}

pub(super) fn repaint(handled: bool, effects: &mut Vec<ChatUiRuntimeEffect>) {
    if handled {
        effects.push(ChatUiRuntimeEffect::Repaint);
    }
}

pub(super) fn selection_text(runtime: &ChatUiRuntime) -> String {
    let state = &runtime.composer.state().text_area;
    let (start, end) = ordered_selection(state.selection.start, state.selection.end);
    state
        .value
        .get(start..end)
        .map_or_else(String::new, ToOwned::to_owned)
}

fn ordered_selection(start: usize, end: usize) -> (usize, usize) {
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

pub(super) fn clipboard_write(runtime: &ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    effects.push(ChatUiRuntimeEffect::Clipboard(
        ChatUiRuntimeClipboard::Write(selection_text(runtime)),
    ));
}
