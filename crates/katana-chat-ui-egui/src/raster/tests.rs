use super::EguiKucView;
use crate::EguiChatController;
use eframe::egui;
use katana_chat_ui::{ChatSession, ChatUiSurface};

mod actions;
mod input;

#[derive(Default)]
struct Controller {
    text: String,
}

impl EguiChatController for Controller {
    fn composer_text_mut(&mut self) -> &mut String {
        &mut self.text
    }
    fn attach(&mut self) {}
    fn submit(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn declared_headless_texture_capability_uploads_real_hidpi_kuc_frame() -> Result<(), String> {
    let surface = ChatUiSurface::from_render_model(&ChatSession::new().render_model());
    let mut view = EguiKucView::default();
    let mut controller = Controller::default();
    let context = egui::Context::default();
    let mut result = Ok(());
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1440.0, 900.0),
        )),
        max_texture_side: Some(4096),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .ok_or("missing root viewport")?
        .native_pixels_per_point = Some(2.0);
    let mut output = context.run_ui(input, |ui| {
        result = view.render(ui, &surface, &mut controller);
    });
    result?;
    assert!(!output.shapes.is_empty());
    verify_headless_kuc_upload(&mut output, [2880, 1800])
}

#[test]
fn oversized_viewport_returns_error_before_texture_allocation() {
    let surface = ChatUiSurface::from_render_model(&ChatSession::new().render_model());
    let mut view = EguiKucView::default();
    let mut controller = Controller::default();
    let context = egui::Context::default();
    let mut result = Ok(());
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2049.0, 600.0),
        )),
        max_texture_side: Some(2048),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        result = view.render(ui, &surface, &mut controller);
    });
    assert_eq!(
        result,
        Err("KUC viewport exceeds the host texture limit".into())
    );
    assert_no_kuc_upload(&mut output, [2049, 600]);
}

pub(super) fn verify_headless_kuc_upload(
    output: &mut egui::FullOutput,
    expected_size: [usize; 2],
) -> Result<(), String> {
    let uploaded = output
        .textures_delta
        .set
        .values()
        .flatten()
        .any(|delta| delta.image.size() == expected_size);
    output.textures_delta.clear();
    if uploaded {
        Ok(())
    } else {
        Err("headless frame did not upload the KUC texture".into())
    }
}

fn assert_no_kuc_upload(output: &mut egui::FullOutput, expected_size: [usize; 2]) {
    let uploaded = output
        .textures_delta
        .set
        .values()
        .flatten()
        .any(|delta| delta.image.size() == expected_size);
    output.textures_delta.clear();
    assert!(!uploaded);
}
