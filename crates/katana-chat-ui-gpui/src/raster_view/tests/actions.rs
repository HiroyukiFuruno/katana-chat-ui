use std::{cell::RefCell, rc::Rc};

use gpui::{Modifiers, TestAppContext, point, px, size};
use katana_chat_ui::{
    ChatSession, ChatUiAction, ChatUiActionMapper, ChatUiHistorySessionSurface, ChatUiSurface,
    kuc::ChatUiRuntimeEffect,
};

use super::super::GpuiKucView;

#[gpui::test]
fn raster_clicks_history_selection_and_stop_through_typed_callbacks(cx: &mut TestAppContext) {
    assert_eq!(run_action_sequence(cx), Ok(()));
}

fn run_action_sequence(cx: &mut TestAppContext) -> Result<(), String> {
    let actions = Rc::new(RefCell::new(Vec::new()));
    let history = history_surface();
    let stopping = stop_surface()?;
    let (view, cx) = mount_view(cx, actions.clone(), history, stopping)?;
    draw(cx);
    click_action(&view, cx, ChatUiAction::ToggleHistory)?;
    draw(cx);
    click_action(
        &view,
        cx,
        ChatUiAction::SelectHistory {
            session_id: "history-7".to_string(),
        },
    )?;
    draw(cx);
    click_action(&view, cx, ChatUiAction::Stop)?;
    if actions.borrow().as_slice() == expected_actions().as_slice() {
        Ok(())
    } else {
        Err(format!("unexpected GPUI actions: {:?}", actions.borrow()))
    }
}

fn mount_view(
    cx: &mut TestAppContext,
    actions: Rc<RefCell<Vec<ChatUiAction>>>,
    history: ChatUiSurface,
    stopping: ChatUiSurface,
) -> Result<(gpui::Entity<GpuiKucView>, &mut gpui::VisualTestContext), String> {
    let surface = ChatUiSurface::from_render_model(&ChatSession::new().render_model());
    let runtime = GpuiKucView::prepare(&surface).map_err(|error| error.to_string())?;
    let (view, cx) = cx.add_window_view(move |_, view_cx| {
        GpuiKucView::from_runtime(
            runtime,
            surface,
            move |effect, _| match effect {
                ChatUiRuntimeEffect::Action(action) => {
                    actions.borrow_mut().push(action.clone());
                    match action {
                        ChatUiAction::ToggleHistory => Some(history.clone()),
                        ChatUiAction::SelectHistory { .. } => Some(stopping.clone()),
                        _ => None,
                    }
                }
                _ => None,
            },
            view_cx,
        )
    });
    Ok((view, cx))
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.simulate_resize(size(px(640.0), px(480.0)));
    cx.update(|window, app| {
        let _ = window.draw(app);
    });
}

fn click_action(
    view: &gpui::Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    expected: ChatUiAction,
) -> Result<(), String> {
    let point = cx.update(|_, app| action_point(view.read(app), &expected))?;
    cx.simulate_click(point, Modifiers::none());
    Ok(())
}

fn action_point(
    view: &GpuiKucView,
    expected: &ChatUiAction,
) -> Result<gpui::Point<gpui::Pixels>, String> {
    let frame = view.frame.as_ref().ok_or("GPUI KUC frame is unavailable")?;
    frame
        .action_hits
        .iter()
        .find(|hit| {
            ChatUiActionMapper::from_host_action(&hit.action)
                .ok()
                .as_ref()
                == Some(expected)
        })
        .map(|hit| point(px(hit.rect.x as f32 + 1.0), px(hit.rect.y as f32 + 1.0)))
        .ok_or_else(|| format!("missing typed KUC action: {expected:?}"))
}

fn history_surface() -> ChatUiSurface {
    let mut surface = ChatUiSurface::from_render_model(&ChatSession::new().render_model());
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

fn expected_actions() -> Vec<ChatUiAction> {
    vec![
        ChatUiAction::ToggleHistory,
        ChatUiAction::SelectHistory {
            session_id: "history-7".to_string(),
        },
        ChatUiAction::Stop,
    ]
}
