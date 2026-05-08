use crate::font::ManualFontStatus;
use crate::state::ManualHostState;
use eframe::egui;
use katana_chat_ui_egui::{EguiChatController, EguiChatView};

pub struct ManualHostApp {
    state: ManualHostState,
}

impl ManualHostApp {
    pub fn new(font_status: ManualFontStatus) -> Self {
        Self {
            state: ManualHostState::new(font_status),
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
        let surface = self.state.surface();
        EguiChatView::render(ui, &surface, self);
    }
}

impl EguiChatController for ManualHostApp {
    fn composer_text_mut(&mut self) -> &mut String {
        self.state.composer_text_mut()
    }

    fn attach(&mut self) {
        self.state.add_sample_attachment();
    }

    fn submit(&mut self) -> Result<(), String> {
        self.state.submit().map_err(|it| it.to_string())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.state.stop().map_err(|it| it.to_string())
    }

    fn new_chat(&mut self) {
        self.state.start_new_chat();
    }

    fn history(&mut self) {
        self.state.open_history();
    }

    fn select_vendor(&mut self, vendor_id: String) {
        self.state.select_vendor(vendor_id);
    }

    fn select_control(&mut self, key: String, value: String) {
        self.state.select_control(key, value);
    }
}
