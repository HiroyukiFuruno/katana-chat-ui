use gpui::{
    Bounds, Context, Entity, EntityInputHandler, Modifiers, Pixels, Render, TestAppContext, Window,
    div, prelude::*, px, size,
};
use katana_chat_ui::{ChatSession, ChatUiSurface};

use super::super::GpuiKucView;
use super::offset_snapshot::OffsetSnapshot;

const OFFSET_X: f32 = 48.0;
const OFFSET_Y: f32 = 36.0;

struct OffsetMount {
    view: Entity<GpuiKucView>,
}

impl Render for OffsetMount {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            div()
                .absolute()
                .left(px(OFFSET_X))
                .top(px(OFFSET_Y))
                .w(px(320.0))
                .h(px(240.0))
                .child(self.view.clone()),
        )
    }
}

#[gpui::test]
fn non_origin_mount_maps_pointer_selection_and_ime_to_the_right_space(cx: &mut TestAppContext) {
    assert_eq!(run_offset_adapter(cx), Ok(()));
}

fn run_offset_adapter(cx: &mut TestAppContext) -> Result<(), String> {
    let surface = ready_surface();
    let runtime = GpuiKucView::prepare(&surface).map_err(|error| error.to_string())?;
    let (mount, cx) = cx.add_window_view(|_, mount_cx| OffsetMount {
        view: mount_cx
            .new(|view_cx| GpuiKucView::from_runtime(runtime, surface, |_, _| None, view_cx)),
    });
    let view = cx.read(|app| mount.read(app).view.clone());
    draw_offset(cx);
    let snapshot = read_snapshot(&view, cx)?;
    assert_mount_origin(snapshot.paint_bounds)?;
    click_composer(cx, snapshot.composer_point);
    draw_offset(cx);
    assert_pointer_selection(&view, cx, snapshot.expected_index)?;
    assert_character_index(&view, cx, snapshot.composer_point, snapshot.expected_index)?;
    assert_ime_bounds(&view, cx, snapshot)
}

fn draw_offset(cx: &mut gpui::VisualTestContext) {
    cx.simulate_resize(size(px(480.0), px(320.0)));
    cx.update(|window, app| {
        let _ = window.draw(app);
    });
}

fn read_snapshot(
    view: &Entity<GpuiKucView>,
    cx: &TestAppContext,
) -> Result<OffsetSnapshot, String> {
    cx.read(|app| OffsetSnapshot::from_view(view.read(app)))
}

fn click_composer(cx: &mut gpui::VisualTestContext, point: gpui::Point<Pixels>) {
    cx.simulate_click(point, Modifiers::none());
}

fn assert_mount_origin(bounds: Bounds<Pixels>) -> Result<(), String> {
    if bounds.left() != px(OFFSET_X) || bounds.top() != px(OFFSET_Y) {
        return Err(format!("expected offset mount, got {bounds:?}"));
    }
    Ok(())
}

fn assert_pointer_selection(
    view: &Entity<GpuiKucView>,
    cx: &TestAppContext,
    expected: usize,
) -> Result<(), String> {
    let selection = cx.read(|app| view.read(app).selection.clone());
    if selection != (expected..expected) {
        return Err(format!(
            "pointer selection was {selection:?}, expected {expected}"
        ));
    }
    Ok(())
}

fn assert_character_index(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    point: gpui::Point<Pixels>,
    expected: usize,
) -> Result<(), String> {
    let index = cx.update(|window, app| {
        view.update(app, |view, view_cx| {
            EntityInputHandler::character_index_for_point(view, point, window, view_cx)
        })
    });
    if index != Some(expected) {
        return Err(format!(
            "character index was {index:?}, expected {expected}"
        ));
    }
    Ok(())
}

fn assert_ime_bounds(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    snapshot: OffsetSnapshot,
) -> Result<(), String> {
    let actual = cx.update(|window, app| {
        view.update(app, |view, view_cx| {
            EntityInputHandler::bounds_for_range(view, 0..0, Bounds::default(), window, view_cx)
        })
    });
    let actual = actual.ok_or("IME range bounds were unavailable")?;
    if actual != snapshot.ime_bounds {
        return Err(format!(
            "IME bounds were {actual:?}, expected {:?}",
            snapshot.ime_bounds
        ));
    }
    Ok(())
}

fn ready_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.draft_mut().set_text("coordinate mapping");
    ChatUiSurface::from_render_model(&session.render_model())
}
