use crate::font::ManualFontStatus;
use crate::state::ManualHostState;
use eframe::egui;
use katana_chat_ui::{CommandLaunchKind, HostActionKind, kuc::ChatUiAction};
use katana_chat_ui_egui::{EguiChatController, EguiChatView};
use std::{env, time::Duration};

const HOST_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub struct ManualHostApp {
    state: ManualHostState,
    view: EguiChatView,
    render_error: Option<String>,
    trace_native_egui: bool,
    last_ime_output: Option<String>,
}

impl ManualHostApp {
    pub fn new(font_status: ManualFontStatus) -> Result<Self, String> {
        Ok(Self {
            state: ManualHostState::new(font_status)?,
            view: EguiChatView::default(),
            render_error: None,
            trace_native_egui: env::var_os("KCU_TRACE_PATH").is_some(),
            last_ime_output: None,
        })
    }
}

impl eframe::App for ManualHostApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint_after(HOST_POLL_INTERVAL);
        self.trace_egui_input(ui);
        let poll = self.state.poll().map(|_| ());
        self.state.record_error(poll);
        let surface = self.state.surface();
        let mut view = std::mem::take(&mut self.view);
        self.render_error = view
            .render(ui, &surface, self)
            .err()
            .or_else(|| self.state.take_error());
        self.view = view;
        self.trace_egui_output(ui);
        let draft = self.state.flush_draft();
        self.state.record_error(draft);
        if let Some(error) = &self.render_error {
            ui.label(error);
        }
    }
}

impl ManualHostApp {
    fn trace_egui_input(&self, ui: &egui::Ui) {
        if !self.trace_native_egui {
            return;
        }
        ui.ctx().input(|input| {
            if !input.events.is_empty() {
                eprintln!("KCU_NATIVE_EGUI_INPUT events={:?}", input.events);
            }
        });
    }

    fn trace_egui_output(&mut self, ui: &egui::Ui) {
        if !self.trace_native_egui {
            return;
        }
        let ime = ui.ctx().output(|output| format!("{:?}", output.ime));
        if self.last_ime_output.as_deref() != Some(ime.as_str()) {
            eprintln!("KCU_NATIVE_EGUI_OUTPUT ime={ime}");
            self.last_ime_output = Some(ime);
        }
    }
}

impl EguiChatController for ManualHostApp {
    fn composer_text_mut(&mut self) -> &mut String {
        self.state.composer_text_mut()
    }

    fn attach(&mut self) {
        self.dispatch_without_result(ChatUiAction::Attach);
    }

    fn submit(&mut self) -> Result<(), String> {
        self.state.dispatch(ChatUiAction::Submit)
    }

    fn stop(&mut self) -> Result<(), String> {
        self.state.dispatch(ChatUiAction::Stop)
    }

    fn new_chat(&mut self) {
        self.dispatch_without_result(ChatUiAction::NewChat);
    }

    fn history(&mut self) {
        self.dispatch_without_result(ChatUiAction::ToggleHistory);
    }

    fn select_vendor(&mut self, vendor_id: String) {
        self.dispatch_without_result(ChatUiAction::SelectVendor { vendor_id });
    }

    fn select_control(&mut self, key: String, value: String) {
        self.dispatch_without_result(ChatUiAction::SelectVendorControl { key, value });
    }

    fn remove_attachment(&mut self, index: usize) -> Result<(), String> {
        self.state
            .dispatch(ChatUiAction::RemoveAttachment { index })
    }

    fn select_history(&mut self, session_id: String) -> Result<(), String> {
        self.state
            .dispatch(ChatUiAction::SelectHistory { session_id })
    }

    fn select_slash_command(
        &mut self,
        entry_id: String,
        kind: CommandLaunchKind,
    ) -> Result<(), String> {
        self.state
            .dispatch(ChatUiAction::SelectSlashCommand { entry_id, kind })
    }

    fn output(&mut self, output_id: u64, kind: HostActionKind) -> Result<(), String> {
        self.state
            .dispatch(ChatUiAction::Output { output_id, kind })
    }

    fn respond_permission(
        &mut self,
        request_id: u64,
        option_id: Option<String>,
    ) -> Result<(), String> {
        self.state.dispatch(ChatUiAction::RespondPermission {
            request_id,
            option_id,
        })
    }
}

impl ManualHostApp {
    fn dispatch_without_result(&mut self, action: ChatUiAction) {
        let result = self.state.dispatch(action);
        self.state.record_error(result);
    }
}
