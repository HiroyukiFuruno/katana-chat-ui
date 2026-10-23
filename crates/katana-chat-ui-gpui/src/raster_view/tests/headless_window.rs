use std::{cell::RefCell, rc::Rc};

use gpui::{Modifiers, TestAppContext, point, px, size};
use katana_chat_ui::{ChatSession, ChatUiAction, ChatUiSurface, kuc::ChatUiRuntimeEffect};

use super::super::GpuiKucView;

#[gpui::test]
fn headless_window_paints_raster_and_routes_pointer_submit(cx: &mut TestAppContext) {
    assert_eq!(run_headless_adapter(cx), Ok(()));
}

fn run_headless_adapter(cx: &mut TestAppContext) -> Result<(), String> {
    let actions = Rc::new(RefCell::new(Vec::new()));
    let surface = ready_surface();
    let runtime = GpuiKucView::prepare(&surface).map_err(|error| error.to_string())?;
    let effect_actions = actions.clone();
    let (view, cx) = cx.add_window_view(|_, view_cx| {
        GpuiKucView::from_runtime(
            runtime,
            surface,
            move |effect, _| {
                record_action(&effect_actions, effect);
                None
            },
            view_cx,
        )
    });
    redraw_narrow(cx);
    let coordinates = assert_narrow_raster(&view, cx)?;
    click_and_submit(cx, coordinates);
    assert_submit(&actions)
}

fn redraw_narrow(cx: &mut gpui::VisualTestContext) {
    cx.simulate_resize(size(px(320.0), px(240.0)));
    cx.update(|window, app| {
        let _ = window.draw(app);
    });
}

fn click_and_submit(cx: &mut gpui::VisualTestContext, coordinates: (i32, i32)) {
    cx.simulate_click(
        point(
            px(coordinates.0 as f32 + 1.0),
            px(coordinates.1 as f32 + 1.0),
        ),
        Modifiers::none(),
    );
    cx.simulate_keystrokes("cmd-enter");
}

fn assert_narrow_raster(
    view: &gpui::Entity<GpuiKucView>,
    cx: &TestAppContext,
) -> Result<(i32, i32), String> {
    cx.read(|app| inspect_narrow_raster(view.read(app)))
}

fn inspect_narrow_raster(view: &GpuiKucView) -> Result<(i32, i32), String> {
    let frame = view
        .frame
        .as_ref()
        .ok_or("GPUI prepaint must retain a KUC frame")?;
    let bounds = frame.composer.bounds;
    let image = view
        .image
        .as_ref()
        .and_then(|image| image.as_bytes(0))
        .ok_or("GPUI paint must create a RenderImage")?;
    validate_raster(
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        frame.rgba.pixels.len() * 4,
        image,
    )?;
    Ok((bounds.x, bounds.y))
}

fn validate_raster(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    rgba_len: usize,
    image: &[u8],
) -> Result<(), String> {
    if width == 0 || height == 0 || x < 0 || y < 0 {
        return Err("narrow layout has invalid composer bounds".into());
    }
    let right = x.saturating_add(i32::try_from(width).map_err(|_| "composer width overflows i32")?);
    if right > 320 {
        return Err("narrow layout exceeds the 320px viewport".into());
    }
    if image.len() != rgba_len || !image.chunks_exact(4).all(|pixel| pixel[3] == u8::MAX) {
        return Err("GPUI RenderImage does not preserve the opaque KUC raster".into());
    }
    Ok(())
}

fn record_action(actions: &Rc<RefCell<Vec<ChatUiAction>>>, effect: ChatUiRuntimeEffect) {
    if let ChatUiRuntimeEffect::Action(action) = effect {
        actions.borrow_mut().push(action);
    }
}

fn assert_submit(actions: &Rc<RefCell<Vec<ChatUiAction>>>) -> Result<(), String> {
    if actions.borrow().as_slice() != [ChatUiAction::Submit] {
        return Err(format!(
            "expected one submit action, got {:?}",
            actions.borrow()
        ));
    }
    Ok(())
}

fn ready_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.draft_mut().set_text("submit this");
    ChatUiSurface::from_render_model(&session.render_model())
}
