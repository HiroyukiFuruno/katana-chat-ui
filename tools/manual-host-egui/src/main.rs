mod app;
mod fixture;
mod font;
mod state;
#[path = "../../manual-host-trace/mod.rs"]
mod trace;

use app::ManualHostApp;
use font::ManualFontInstaller;

const HARNESS_WINDOW_WIDTH: f32 = 1280.0;
const HARNESS_WINDOW_HEIGHT: f32 = 900.0;

fn main() -> eframe::Result<()> {
    let fixture = match fixture::load_from_args() {
        Ok(fixture) => fixture,
        Err(error) => fatal_fixture_startup(&error),
    };
    let (width, height) = fixture
        .as_ref()
        .map_or((HARNESS_WINDOW_WIDTH, HARNESS_WINDOW_HEIGHT), |it| {
            (it.width as f32, it.height as f32)
        });
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([width, height]),
        ..Default::default()
    };
    eframe::run_native(
        "katana-chat-ui harness egui",
        options,
        Box::new(|context| {
            let font_status = ManualFontInstaller::install(&context.egui_ctx);
            let app: Box<dyn eframe::App> = match fixture {
                Some(fixture) => Box::new(fixture::FixtureApp::new(fixture.surface)),
                None => Box::new(ManualHostApp::new(font_status).map_err(std::io::Error::other)?),
            };
            Ok(app)
        }),
    )
}

fn fatal_fixture_startup(error: &str) -> ! {
    eprintln!("manual-host-egui fixture failed to start: {error}");
    std::process::exit(1)
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
