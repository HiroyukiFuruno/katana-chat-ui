use crate::raster_view::GpuiKucView;
use gpui::{Context, KeyDownEvent, Window};
use katana_chat_ui::kuc::ChatUiRuntimeInput;

pub(super) fn apply(
    view: &mut GpuiKucView,
    event: &KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<GpuiKucView>,
) -> bool {
    if !event.keystroke.modifiers.platform {
        return false;
    }
    match event.keystroke.key.as_str() {
        "c" => view.dispatch_input(ChatUiRuntimeInput::ClipboardCopy, window, cx),
        "x" => view.dispatch_input(ChatUiRuntimeInput::ClipboardCut, window, cx),
        "v" => paste(view, window, cx),
        _ => return false,
    }
    true
}

fn paste(view: &mut GpuiKucView, window: &mut Window, cx: &mut Context<GpuiKucView>) {
    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
        view.dispatch_input(ChatUiRuntimeInput::ClipboardPaste(text), window, cx);
    }
}
