mod app;
mod font;
mod state;

use app::ManualHostApp;
use font::ManualFontInstaller;

const HARNESS_WINDOW_WIDTH: f32 = 1280.0;
const HARNESS_WINDOW_HEIGHT: f32 = 900.0;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([HARNESS_WINDOW_WIDTH, HARNESS_WINDOW_HEIGHT]),
        ..Default::default()
    };
    eframe::run_native(
        "katana-chat-ui harness egui",
        options,
        Box::new(|context| {
            let font_status = ManualFontInstaller::install(&context.egui_ctx);
            Ok(Box::new(ManualHostApp::new(font_status)))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_WINDOW_HEIGHT, HARNESS_WINDOW_WIDTH};

    #[test]
    fn harness_window_size_matches_floem_baseline() {
        assert_eq!(HARNESS_WINDOW_WIDTH, 1280.0);
        assert_eq!(HARNESS_WINDOW_HEIGHT, 900.0);
    }
}
