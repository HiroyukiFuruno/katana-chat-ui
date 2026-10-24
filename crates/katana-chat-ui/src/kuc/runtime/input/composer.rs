use super::super::state::ChatUiRuntime;
use super::super::types::{
    ChatUiRuntimeCompositionPhase, ChatUiRuntimeEffect, ChatUiRuntimeInput as Input,
};
use super::effects;
use katana_ui_core::atom::{TextAreaAction, TextAreaCompositionPhase, TextAreaSelection};
use katana_ui_core::text_surface::{TextSurfaceAction, TextSurfaceClipboardOperation};

pub(super) fn dispatch(
    runtime: &mut ChatUiRuntime,
    input: Input,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    match input {
        Input::Text(value) | Input::ClipboardPaste(value) => text(runtime, value, effects),
        Input::Key(key) => super::key::apply(runtime, key, effects),
        Input::ImeComposition {
            phase,
            preedit,
            caret,
        } => composition(runtime, phase, preedit, caret, effects),
        Input::ImeCommit(value) => ime_commit(runtime, value, effects),
        Input::ImeCommitRange {
            start_byte,
            end_byte,
            text,
        } => ime_commit_range(runtime, start_byte, end_byte, text, effects),
        Input::ImeCancel => ime_cancel(runtime, effects),
        Input::FocusLost => focus_lost(runtime, effects),
        Input::ClipboardCopy => copy(runtime, effects),
        Input::ClipboardCut => cut(runtime, effects),
        Input::PointerPress { .. }
        | Input::PointerDrag { .. }
        | Input::PointerRelease
        | Input::Scroll { .. } => {}
    }
}

pub(super) fn text(
    runtime: &mut ChatUiRuntime,
    value: String,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    if !focused(runtime) {
        return;
    }
    apply(
        runtime,
        TextSurfaceAction::TextArea(TextAreaAction::Type(value)),
        effects,
    );
}

pub(super) fn composition(
    runtime: &mut ChatUiRuntime,
    phase: ChatUiRuntimeCompositionPhase,
    preedit: String,
    caret: usize,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    if !focused(runtime) {
        return;
    }
    let phase = match phase {
        ChatUiRuntimeCompositionPhase::Start => TextAreaCompositionPhase::Start,
        ChatUiRuntimeCompositionPhase::Update => TextAreaCompositionPhase::Update,
    };
    let action = TextAreaAction::composition(phase, preedit, caret);
    apply(runtime, TextSurfaceAction::TextArea(action), effects);
}

pub(super) fn ime_commit(
    runtime: &mut ChatUiRuntime,
    value: String,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    if !focused(runtime) {
        return;
    }
    apply(
        runtime,
        TextSurfaceAction::TextArea(TextAreaAction::ime_commit(value)),
        effects,
    );
}

pub(super) fn ime_commit_range(
    runtime: &mut ChatUiRuntime,
    start_byte: usize,
    end_byte: usize,
    value: String,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    if !focused(runtime) {
        return;
    }
    apply(
        runtime,
        TextSurfaceAction::TextArea(TextAreaAction::Select(TextAreaSelection {
            start: start_byte,
            end: end_byte,
        })),
        effects,
    );
    ime_commit(runtime, value, effects);
}

pub(super) fn ime_cancel(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    apply(runtime, TextSurfaceAction::CancelComposition, effects);
}

pub(super) fn focus_lost(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    apply(runtime, TextSurfaceAction::SetFocus(false), effects);
}

pub(super) fn copy(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    if !focused(runtime) {
        return;
    }
    request_clipboard(runtime, TextSurfaceClipboardOperation::Copy, effects);
}

pub(super) fn cut(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    if !focused(runtime) {
        return;
    }
    if request_clipboard(runtime, TextSurfaceClipboardOperation::Cut, effects) {
        apply(
            runtime,
            TextSurfaceAction::TextArea(TextAreaAction::Type(String::new())),
            effects,
        );
    }
}

fn request_clipboard(
    runtime: &mut ChatUiRuntime,
    operation: TextSurfaceClipboardOperation,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) -> bool {
    let outcome = runtime
        .composer
        .apply_action(TextSurfaceAction::ClipboardRequest(operation));
    if outcome.handled {
        effects::clipboard_write(runtime, effects);
    }
    effects::repaint(outcome.handled, effects);
    outcome.handled
}

pub(super) fn apply(
    runtime: &mut ChatUiRuntime,
    action: TextSurfaceAction,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    let outcome = runtime.composer.apply_action(action);
    effects::collect(&outcome.events, true, effects);
    effects::repaint(outcome.handled, effects);
}

fn focused(runtime: &ChatUiRuntime) -> bool {
    runtime.composer.state().text_area.focused
}
