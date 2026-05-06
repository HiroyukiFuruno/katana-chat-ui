mod app;
mod composer_render;
mod font;
mod ollama;
mod output_render;
mod render;
mod state;

use app::ManualHostApp;
use font::ManualFontInstaller;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "katana-chat-ui 手動確認",
        options,
        Box::new(|context| {
            let font_status = ManualFontInstaller::install(&context.egui_ctx);
            Ok(Box::new(ManualHostApp::new(font_status)))
        }),
    )
}
