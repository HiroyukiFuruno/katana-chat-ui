use super::super::EguiKucView;
use super::{Controller, verify_headless_kuc_upload};
use crate::EguiChatController;
use eframe::egui;
use katana_chat_ui::{
    ChatSession, ChatUiAction, ChatUiActionMapper, ChatUiHistorySessionSurface, ChatUiSurface,
    kuc::ChatUiRuntimeViewport,
};
const VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(640, 480, 1.0);
#[derive(Default)]
struct RecordingController {
    inner: Controller,
    actions: Vec<ChatUiAction>,
}
impl EguiChatController for RecordingController {
    fn composer_text_mut(&mut self) -> &mut String {
        self.inner.composer_text_mut()
    }

    fn attach(&mut self) {}

    fn submit(&mut self) -> Result<(), String> {
        self.inner.submit()
    }

    fn stop(&mut self) -> Result<(), String> {
        self.actions.push(ChatUiAction::Stop);
        Ok(())
    }

    fn history(&mut self) {
        self.actions.push(ChatUiAction::ToggleHistory);
    }

    fn select_history(&mut self, session_id: String) -> Result<(), String> {
        self.actions
            .push(ChatUiAction::SelectHistory { session_id });
        Ok(())
    }
}
#[test]
fn raster_clicks_history_selection_and_stop_through_typed_callbacks() -> Result<(), String> {
    let context = egui::Context::default();
    let mut view = EguiKucView::default();
    let mut controller = RecordingController::default();
    let closed = surface();
    exercise(
        &context,
        &mut view,
        &closed,
        &mut controller,
        ChatUiAction::ToggleHistory,
    )?;
    let history = history_surface();
    let selected = ChatUiAction::SelectHistory {
        session_id: "history-7".to_string(),
    };
    exercise(&context, &mut view, &history, &mut controller, selected)?;
    let stopping = stop_surface()?;
    exercise(
        &context,
        &mut view,
        &stopping,
        &mut controller,
        ChatUiAction::Stop,
    )?;
    assert_eq!(controller.actions, expected_actions());
    Ok(())
}
fn exercise(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
    action: ChatUiAction,
) -> Result<(), String> {
    render(context, view, surface, controller, Vec::new())?;
    click_action(context, view, surface, controller, action)
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
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
    action: ChatUiAction,
) -> Result<(), String> {
    let point = action_point(view, surface, &action)?;
    render(context, view, surface, controller, click(point))
}

fn action_point(
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    expected: &ChatUiAction,
) -> Result<egui::Pos2, String> {
    let runtime = view
        .runtime
        .as_mut()
        .ok_or("egui KUC runtime is unavailable")?;
    let frame = runtime
        .render(surface, VIEWPORT)
        .map_err(|error| error.to_string())?;
    frame
        .action_hits
        .iter()
        .find(|hit| {
            ChatUiActionMapper::from_host_action(&hit.action)
                .ok()
                .as_ref()
                == Some(expected)
        })
        .map(|hit| egui::pos2(hit.rect.x as f32 + 1.0, hit.rect.y as f32 + 1.0))
        .ok_or_else(|| format!("missing typed KUC action: {expected:?}"))
}

fn render(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
    events: Vec<egui::Event>,
) -> Result<(), String> {
    let mut result = Ok(());
    let mut output = context.run_ui(raw_input(events), |ui| {
        result = view.render(ui, surface, controller);
    });
    result?;
    verify_headless_kuc_upload(&mut output, [640, 480])
}

fn raw_input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(640.0, 480.0),
        )),
        max_texture_side: Some(4096),
        events,
        ..Default::default()
    }
}

fn click(point: egui::Pos2) -> Vec<egui::Event> {
    vec![button(point, true), button(point, false)]
}

fn button(point: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: point,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    }
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
