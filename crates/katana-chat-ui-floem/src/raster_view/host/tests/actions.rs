use std::{cell::RefCell, rc::Rc};

use floem::{
    event::Event,
    kurbo::Point,
    pointer::{PointerButton, PointerInputEvent},
    reactive::{RwSignal, SignalUpdate},
};
use katana_chat_ui::{
    ChatSession, ChatUiAction, ChatUiActionMapper, ChatUiHistorySessionSurface, ChatUiSurface,
};

use super::{
    super::{super::events::event_input, RasterChatView},
    fixture::{callbacks, viewport},
};

#[test]
fn raster_clicks_history_selection_and_stop_through_typed_callbacks() -> Result<(), String> {
    let actions = Rc::new(RefCell::new(Vec::new()));
    let drafts = Rc::new(RefCell::new(Vec::new()));
    let surface = RwSignal::new(surface());
    let mut view = RasterChatView::new(
        surface,
        RwSignal::new(String::new()),
        1.0,
        callbacks(actions.clone(), drafts),
    )
    .map_err(|error| error.to_string())?;
    view.render(viewport()).map_err(|error| error.to_string())?;
    click_action(&mut view, ChatUiAction::ToggleHistory)?;
    surface.set(history_surface());
    view.render(viewport()).map_err(|error| error.to_string())?;
    click_action(
        &mut view,
        ChatUiAction::SelectHistory {
            session_id: "history-7".to_string(),
        },
    )?;
    surface.set(stop_surface()?);
    view.render(viewport()).map_err(|error| error.to_string())?;
    click_action(&mut view, ChatUiAction::Stop)?;
    assert_eq!(actions.borrow().as_slice(), expected_actions().as_slice());
    Ok(())
}

fn surface() -> ChatUiSurface {
    ChatUiSurface::from_render_model(&ChatSession::new().render_model())
}

fn history_surface() -> ChatUiSurface {
    let mut surface = surface();
    surface.history_panel.visible = true;
    surface
        .history_panel
        .sessions
        .push(ChatUiHistorySessionSurface {
            session_id: "history-7".to_string(),
            title: "Saved chat".to_string(),
            provider_id: "katanagent".to_string(),
            provider_label: "KatanAgent".to_string(),
            updated_at_label: "now".to_string(),
            preview: "history preview".to_string(),
        });
    surface
}

fn stop_surface() -> Result<ChatUiSurface, String> {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session
        .start_assistant_stream("working")
        .map_err(|error| error.to_string())?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn click_action(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
    expected: ChatUiAction,
) -> Result<(), String> {
    let point = action_point(view, &expected)?;
    let event = Event::PointerDown(PointerInputEvent {
        pos: Point::new(f64::from(point.0), f64::from(point.1)),
        button: PointerButton::Primary,
        modifiers: Default::default(),
        count: 1,
    });
    let input = event_input(&event, false).ok_or("Floem pointer was not mapped")?;
    view.dispatch(input);
    Ok(())
}

fn action_point(
    view: &RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
    expected: &ChatUiAction,
) -> Result<(f32, f32), String> {
    let frame = view
        .frame
        .as_ref()
        .ok_or("Floem KUC frame is unavailable")?;
    frame
        .action_hits
        .iter()
        .find(|hit| {
            ChatUiActionMapper::from_host_action(&hit.action)
                .ok()
                .as_ref()
                == Some(expected)
        })
        .map(|hit| hit.center_point())
        .ok_or_else(|| format!("missing typed KUC action: {expected:?}"))
}

fn expected_actions() -> Vec<ChatUiAction> {
    vec![
        ChatUiAction::ToggleHistory,
        ChatUiAction::SelectHistory {
            session_id: "history-7".to_string(),
        },
        ChatUiAction::Stop,
    ]
}
