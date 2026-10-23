use super::super::state::ChatUiRuntime;
use super::super::types::{ChatUiRuntimeEffect, ChatUiRuntimeKey};
use super::composer;
use katana_ui_core::atom::{
    TextAreaAction, TextAreaCaretMove, TextAreaKey, TextAreaKeyChord, TextAreaSelection,
};
use katana_ui_core::text_surface::TextSurfaceAction;

pub(super) fn apply(
    runtime: &mut ChatUiRuntime,
    key: ChatUiRuntimeKey,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    if !runtime.composer.state().text_area.focused {
        return;
    }
    match key {
        ChatUiRuntimeKey::Enter { shift, command } => enter(runtime, shift, command, effects),
        ChatUiRuntimeKey::Backspace => {
            text_action(runtime, TextAreaAction::DeleteBackward, effects)
        }
        ChatUiRuntimeKey::Delete => delete_forward(runtime, effects),
        ChatUiRuntimeKey::ArrowLeft => {
            move_caret(runtime, TextAreaCaretMove::PreviousGrapheme, effects)
        }
        ChatUiRuntimeKey::ArrowRight => {
            move_caret(runtime, TextAreaCaretMove::NextGrapheme, effects)
        }
        ChatUiRuntimeKey::Home => move_caret(runtime, TextAreaCaretMove::Start, effects),
        ChatUiRuntimeKey::End => move_caret(runtime, TextAreaCaretMove::End, effects),
        ChatUiRuntimeKey::CommandA => select_all(runtime, effects),
    }
}

fn enter(
    runtime: &mut ChatUiRuntime,
    shift: bool,
    command: bool,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    let key = TextAreaKeyChord {
        key: TextAreaKey::Enter,
        shift,
        primary_modifier: command,
    };
    composer::apply(runtime, TextSurfaceAction::Key(key), effects);
}

fn delete_forward(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    move_caret(runtime, TextAreaCaretMove::NextGrapheme, effects);
    text_action(runtime, TextAreaAction::DeleteBackward, effects);
}

fn move_caret(
    runtime: &mut ChatUiRuntime,
    movement: TextAreaCaretMove,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    text_action(runtime, TextAreaAction::MoveCaret(movement), effects);
}

fn select_all(runtime: &mut ChatUiRuntime, effects: &mut Vec<ChatUiRuntimeEffect>) {
    let end = runtime.composer.state().text_area.value.len();
    text_action(
        runtime,
        TextAreaAction::Select(TextAreaSelection { start: 0, end }),
        effects,
    );
}

fn text_action(
    runtime: &mut ChatUiRuntime,
    action: TextAreaAction,
    effects: &mut Vec<ChatUiRuntimeEffect>,
) {
    composer::apply(runtime, TextSurfaceAction::TextArea(action), effects);
}
