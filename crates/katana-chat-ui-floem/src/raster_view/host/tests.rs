mod actions;
mod fixture;
mod repeat;

use std::{cell::RefCell, rc::Rc};

use floem::{
    event::Event,
    kurbo::Point,
    pointer::{PointerButton, PointerInputEvent},
    reactive::RwSignal,
};
use katana_chat_ui::ChatUiAction;

use super::{super::events::event_input, RasterChatView};
use fixture::{assert_preview, callbacks, ready_surface, view_with_draft, viewport};

#[test]
fn headless_refreshes_the_kuc_raster_before_native_paint() -> Result<(), String> {
    let mut view = view_with_draft("headless draft")?;

    view.render(viewport()).map_err(|error| error.to_string())?;
    let frame = view.frame.as_ref().ok_or("KUC frame was not retained")?;
    assert_eq!(frame.rgba.width, 640);
    assert_eq!(frame.rgba.height, 480);
    assert!(view.image.is_some());
    Ok(())
}

#[test]
fn headless_view_renders_floem_image_and_routes_native_events() -> Result<(), String> {
    let actions = Rc::new(RefCell::new(Vec::new()));
    let drafts = Rc::new(RefCell::new(Vec::new()));
    let surface = RwSignal::new(ready_surface());
    let mut view = RasterChatView::new(
        surface,
        RwSignal::new("submit this".to_string()),
        1.0,
        callbacks(actions.clone(), drafts.clone()),
    )
    .map_err(|error| error.to_string())?;
    view.render(viewport()).map_err(|error| error.to_string())?;
    assert_preview(&view)?;
    focus_and_commit_ime(&mut view)?;
    select_vendor(&mut view)?;
    submit(&mut view)?;
    assert_actions(&actions)?;
    assert_drafts(&drafts)
}

fn focus_and_commit_ime(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
) -> Result<(), String> {
    focus(view)?;
    dispatch_event(view, Event::ImeCommit("日本".to_string()))
}

fn focus(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
) -> Result<(), String> {
    let point = view
        .frame
        .as_ref()
        .map(|frame| {
            (
                frame.composer.bounds.x as f32,
                frame.composer.bounds.y as f32,
            )
        })
        .ok_or("missing composer frame")?;
    dispatch_event(view, pointer(point.0, point.1))
}

fn select_vendor(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
) -> Result<(), String> {
    let point = node_center(view, "chat.chrome.provider-selector")?;
    dispatch_event(view, pointer(point.0, point.1))?;
    view.render(viewport()).map_err(|error| error.to_string())?;
    let point = action_center(view, "codex-cli")?;
    dispatch_event(view, pointer(point.0, point.1))
}

fn submit(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
) -> Result<(), String> {
    let point = action_center(view, "Submit")?;
    dispatch_event(view, pointer(point.0, point.1))
}

fn dispatch_event(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
    event: Event,
) -> Result<(), String> {
    let input = event_input(&event, false).ok_or("Floem event was not mapped")?;
    view.dispatch(input);
    Ok(())
}

fn node_center(
    view: &RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
    id: &str,
) -> Result<(f32, f32), String> {
    let frame = view.frame.as_ref().ok_or("missing Floem frame")?;
    frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == id)
        .map(|hit| hit.rect.center_point())
        .ok_or_else(|| format!("missing Floem node hit: {id}"))
}

fn action_center(
    view: &RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
    value: &str,
) -> Result<(f32, f32), String> {
    let frame = view.frame.as_ref().ok_or("missing Floem frame")?;
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(value))
        .map(|hit| hit.center_point())
        .ok_or_else(|| format!("missing Floem action hit: {value}"))
}

fn pointer(x: f32, y: f32) -> Event {
    Event::PointerDown(PointerInputEvent {
        pos: Point::new(f64::from(x), f64::from(y)),
        button: PointerButton::Primary,
        modifiers: Default::default(),
        count: 1,
    })
}

fn assert_actions(actions: &Rc<RefCell<Vec<ChatUiAction>>>) -> Result<(), String> {
    let expected = [
        ChatUiAction::SelectVendor {
            vendor_id: "codex-cli".to_string(),
        },
        ChatUiAction::Submit,
    ];
    if actions.borrow().as_slice() != expected {
        return Err(format!(
            "unexpected Floem typed actions: {:?}",
            actions.borrow()
        ));
    }
    Ok(())
}

fn assert_drafts(drafts: &Rc<RefCell<Vec<String>>>) -> Result<(), String> {
    if drafts.borrow().iter().any(|draft| draft.contains("日本")) {
        return Ok(());
    }
    Err("Floem IME event did not reach the draft callback".into())
}
