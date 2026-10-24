mod event_fixture;

use super::super::EguiKucView;
use super::verify_headless_kuc_upload;
use crate::EguiChatController;
use eframe::egui;
use katana_chat_ui::{ChatSession, ChatUiSurface, kuc::ChatUiRuntimeViewport};

use event_fixture::{enter, pointer_click, raw_input};

const VIEWPORT_WIDTH: f32 = 640.0;
const VIEWPORT_HEIGHT: f32 = 480.0;

#[derive(Default)]
struct RecordingController {
    draft: String,
    submit_count: usize,
}

impl EguiChatController for RecordingController {
    fn composer_text_mut(&mut self) -> &mut String {
        &mut self.draft
    }

    fn attach(&mut self) {}

    fn submit(&mut self) -> Result<(), String> {
        self.submit_count += 1;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn context_frame_routes_composer_ime_enter_and_command_submit() -> Result<(), String> {
    let context = egui::Context::default();
    let surface = ready_surface("");
    let mut view = EguiKucView::default();
    let mut controller = RecordingController::default();
    render_frame(&context, &mut view, &surface, &mut controller, Vec::new())?;
    let composer = composer_point(&mut view, &surface)?;
    focus_composer(&context, &mut view, &surface, &mut controller, composer)?;
    commit_ime(&context, &mut view, &surface, &mut controller)?;
    assert_eq!(controller.draft, "日本");
    let surface = ready_surface(&controller.draft);
    insert_newline(&context, &mut view, &surface, &mut controller)?;
    assert_eq!(controller.draft, "日本\n");
    let surface = ready_surface(&controller.draft);
    assert_submit_enabled(&surface)?;
    submit_once(&context, &mut view, &surface, &mut controller)?;
    assert_eq!(controller.submit_count, 1);
    Ok(())
}

fn ready_surface(draft: &str) -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.draft_mut().set_text(draft);
    ChatUiSurface::from_render_model(&session.render_model())
}

fn composer_point(view: &mut EguiKucView, surface: &ChatUiSurface) -> Result<egui::Pos2, String> {
    let runtime = view
        .runtime
        .as_mut()
        .ok_or("egui view did not retain a KUC runtime")?;
    let frame = runtime
        .render(
            surface,
            ChatUiRuntimeViewport::new(VIEWPORT_WIDTH as u32, VIEWPORT_HEIGHT as u32, 1.0),
        )
        .map_err(|error| error.to_string())?;
    Ok(egui::pos2(
        frame.composer.bounds.x as f32 + 1.0,
        frame.composer.bounds.y as f32 + 1.0,
    ))
}

fn focus_composer(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
    composer: egui::Pos2,
) -> Result<(), String> {
    render_frame(context, view, surface, controller, pointer_click(composer))?;
    render_frame(context, view, surface, controller, Vec::new())
}

fn commit_ime(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
) -> Result<(), String> {
    render_frame(
        context,
        view,
        surface,
        controller,
        vec![egui::Event::Ime(egui::ImeEvent::Commit("日本".to_string()))],
    )
}

fn insert_newline(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
) -> Result<(), String> {
    render_frame(
        context,
        view,
        surface,
        controller,
        vec![enter(false, true), enter(false, false)],
    )
}

fn submit_once(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut RecordingController,
) -> Result<(), String> {
    render_frame(
        context,
        view,
        surface,
        controller,
        vec![enter(true, true), enter(true, true), enter(true, false)],
    )
}

fn render_frame(
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
    // RawInputで入力経路だけを検査するため、GPUを使わずdeltaを検査後に消費する。
    verify_headless_kuc_upload(&mut output, [640, 480])
}

fn assert_submit_enabled(surface: &ChatUiSurface) -> Result<(), String> {
    if surface.composer.send_enabled {
        return Ok(());
    }
    Err("host draft surface did not enable KUC submit".into())
}
