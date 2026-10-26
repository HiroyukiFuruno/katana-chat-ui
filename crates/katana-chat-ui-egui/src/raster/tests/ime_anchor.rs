use super::{Controller, EguiKucView};
use eframe::egui;
use katana_chat_ui::{ChatSession, ChatUiSurface, kuc::ChatUiRuntimeViewport};

#[test]
fn focused_empty_composer_exposes_ime_caret_inside_editor() -> Result<(), String> {
    let surface = ChatUiSurface::from_render_model(&ChatSession::new().render_model());
    let context = egui::Context::default();
    let mut view = EguiKucView::default();
    let mut controller = Controller::default();
    render(&context, &mut view, &surface, &mut controller, Vec::new())?;
    let events = composer_click_events(&mut view, &surface)?;
    render(&context, &mut view, &surface, &mut controller, events)?;
    let output = render(&context, &mut view, &surface, &mut controller, Vec::new())?;
    assert_ime_caret_is_inside_editor(&output)?;
    assert!(controller.text.is_empty());
    Ok(())
}

fn composer_click_events(
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
) -> Result<Vec<egui::Event>, String> {
    let frame = view
        .runtime
        .as_mut()
        .ok_or("KUC runtime not initialized")?
        .render(surface, ChatUiRuntimeViewport::new(640, 480, 1.0))
        .map_err(|error| error.to_string())?;
    let point = egui::pos2(
        frame.composer.bounds.x as f32 + 12.0,
        frame.composer.bounds.y as f32 + 12.0,
    );
    Ok([true, false]
        .into_iter()
        .map(|pressed| egui::Event::PointerButton {
            pos: point,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        })
        .collect())
}

fn assert_ime_caret_is_inside_editor(output: &egui::FullOutput) -> Result<(), String> {
    let ime = output
        .platform_output
        .ime
        .ok_or("focused composer has no IME output")?;
    assert!(
        ime.rect.contains_rect(ime.cursor_rect),
        "IME cursor outside editor: {ime:?}"
    );
    assert!(ime.cursor_rect.left() > ime.rect.left());
    assert!(ime.cursor_rect.top() > ime.rect.top());
    assert!(ime.cursor_rect.height() > 1.0);
    Ok(())
}

fn render(
    context: &egui::Context,
    view: &mut EguiKucView,
    surface: &ChatUiSurface,
    controller: &mut Controller,
    events: Vec<egui::Event>,
) -> Result<egui::FullOutput, String> {
    let mut result = Ok(());
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 480.0),
            )),
            max_texture_side: Some(4096),
            events,
            ..Default::default()
        },
        |ui| result = view.render(ui, surface, controller),
    );
    result?;
    output.textures_delta.clear();
    Ok(output)
}
