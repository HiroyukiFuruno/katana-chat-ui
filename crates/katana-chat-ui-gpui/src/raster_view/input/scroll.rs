use crate::raster_view::GpuiKucView;
use gpui::{Context, ScrollDelta, ScrollWheelEvent, Window};
use katana_chat_ui::kuc::ChatUiRuntimeInput;

pub(crate) fn apply(
    view: &mut GpuiKucView,
    event: &ScrollWheelEvent,
    window: &mut Window,
    cx: &mut Context<GpuiKucView>,
) {
    let (delta_x, delta_y) = delta(event.delta);
    let Some(position) = view.local_point(event.position) else {
        return;
    };
    view.dispatch_input(
        ChatUiRuntimeInput::Scroll {
            x: f32::from(position.x),
            y: f32::from(position.y),
            delta_x,
            delta_y,
        },
        window,
        cx,
    );
}

fn delta(delta: ScrollDelta) -> (i32, i32) {
    match delta {
        ScrollDelta::Pixels(point) => (
            f32::from(point.x).round() as i32,
            f32::from(point.y).round() as i32,
        ),
        ScrollDelta::Lines(point) => (point.x.round() as i32, point.y.round() as i32),
    }
}
