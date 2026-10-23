use crate::raster_view::GpuiKucView;
use gpui::{Context, MouseMoveEvent, MouseUpEvent, Window};
use katana_chat_ui::kuc::ChatUiRuntimeInput;

pub(crate) fn drag(
    view: &mut GpuiKucView,
    event: &MouseMoveEvent,
    window: &mut Window,
    cx: &mut Context<GpuiKucView>,
) {
    let Some(position) = view.local_point(event.position) else {
        return;
    };
    view.dispatch_input(
        ChatUiRuntimeInput::PointerDrag {
            x: f32::from(position.x),
            y: f32::from(position.y),
        },
        window,
        cx,
    );
}

pub(crate) fn release(
    view: &mut GpuiKucView,
    _: &MouseUpEvent,
    window: &mut Window,
    cx: &mut Context<GpuiKucView>,
) {
    view.dispatch_input(ChatUiRuntimeInput::PointerRelease, window, cx);
}
