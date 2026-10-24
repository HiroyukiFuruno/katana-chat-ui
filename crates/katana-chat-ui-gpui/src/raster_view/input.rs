mod clipboard;
mod composition;
mod ime;
mod pointer;
mod scroll;
#[cfg(test)]
mod tests;
mod utf16;

use gpui::{Context, KeyDownEvent, MouseDownEvent, Window};
use katana_chat_ui::kuc::{ChatUiRuntimeInput, ChatUiRuntimeKey};

pub(super) use composition::CompositionState;

use super::GpuiKucView;

pub(super) fn pointer_press(
    view: &mut GpuiKucView,
    event: &MouseDownEvent,
    window: &mut Window,
    cx: &mut Context<GpuiKucView>,
) {
    window.focus(&view.focus_handle);
    let Some(position) = view.local_point(event.position) else {
        return;
    };
    view.dispatch_input(
        ChatUiRuntimeInput::PointerPress {
            x: f32::from(position.x),
            y: f32::from(position.y),
            extend_selection: false,
        },
        window,
        cx,
    );
}

pub(super) use pointer::{drag as pointer_drag, release as pointer_release};

pub(super) use scroll::apply as scroll;

pub(super) fn key_down(
    view: &mut GpuiKucView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<GpuiKucView>,
) {
    if clipboard::apply(view, event, window, cx) {
        return;
    }
    if command_enter_repeat(event) {
        return;
    }
    let Some(key) = runtime_key(event) else {
        return;
    };
    view.dispatch_input(ChatUiRuntimeInput::Key(key), window, cx);
}

fn command_enter_repeat(event: &KeyDownEvent) -> bool {
    event.is_held && event.keystroke.modifiers.platform && event.keystroke.key == "enter"
}

fn runtime_key(event: &KeyDownEvent) -> Option<ChatUiRuntimeKey> {
    let modifiers = &event.keystroke.modifiers;
    match event.keystroke.key.as_str() {
        "enter" => Some(ChatUiRuntimeKey::Enter {
            shift: modifiers.shift,
            command: modifiers.platform,
        }),
        "backspace" => Some(ChatUiRuntimeKey::Backspace),
        "delete" => Some(ChatUiRuntimeKey::Delete),
        "left" => Some(ChatUiRuntimeKey::ArrowLeft),
        "right" => Some(ChatUiRuntimeKey::ArrowRight),
        "home" => Some(ChatUiRuntimeKey::Home),
        "end" => Some(ChatUiRuntimeKey::End),
        "a" if modifiers.platform => Some(ChatUiRuntimeKey::CommandA),
        _ => None,
    }
}
