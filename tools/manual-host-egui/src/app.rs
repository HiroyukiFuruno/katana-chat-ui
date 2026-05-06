use crate::font::ManualFontStatus;
use crate::render::ManualHostRenderer;
use crate::state::ManualHostState;
use eframe::egui;

pub struct ManualHostApp {
    state: ManualHostState,
    last_error: Option<String>,
}

impl ManualHostApp {
    pub fn new(font_status: ManualFontStatus) -> Self {
        Self {
            state: ManualHostState::new(font_status),
            last_error: None,
        }
    }

    fn handle_os_drops(&mut self, context: &egui::Context) {
        let dropped_files = context.input(|input| input.raw.dropped_files.clone());
        for file in dropped_files {
            if let Some(path) = file.path {
                self.state.add_os_drop(path.display().to_string());
            }
        }
    }
}

impl eframe::App for ManualHostApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_os_drops(ui.ctx());
        ManualHostRenderer::render(ui, &mut self.state, &mut self.last_error);
    }
}
