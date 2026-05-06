use crate::state::ManualHostState;
use eframe::egui;

pub struct ManualComposerRenderer;

impl ManualComposerRenderer {
    pub fn render(ui: &mut egui::Ui, state: &mut ManualHostState, last_error: &mut Option<String>) {
        ui.label("入力欄（composer）");
        ui.add(
            egui::TextEdit::multiline(state.composer_text_mut())
                .desired_rows(4)
                .hint_text("ここに確認用の文章を入力します"),
        );
        Self::attachment_controls(ui, state);
        Self::send_controls(ui, state, last_error);
        Self::ollama_controls(ui, state, last_error);
        ui.label("ファイルをこのウィンドウへ drop しても添付扱いになります。");
    }

    fn attachment_controls(ui: &mut egui::Ui, state: &mut ManualHostState) {
        ui.horizontal(|ui| {
            if ui.button("サンプル添付を追加（attach）").clicked() {
                state.add_sample_attachment();
            }
            ui.text_edit_singleline(state.path_drop_text_mut());
            if ui.button("path drop として追加").clicked() {
                state.add_path_drop();
            }
        });
    }

    fn send_controls(
        ui: &mut egui::Ui,
        state: &mut ManualHostState,
        last_error: &mut Option<String>,
    ) {
        ui.horizontal(|ui| {
            if ui.button("送信（send）").clicked() {
                Self::store_result(state.submit(), last_error);
            }
            if ui.button("停止（stop）").clicked() {
                Self::store_result(state.stop(), last_error);
            }
        });
    }

    fn ollama_controls(
        ui: &mut egui::Ui,
        state: &mut ManualHostState,
        last_error: &mut Option<String>,
    ) {
        ui.group(|ui| {
            ui.label("Ollama local LLM 確認");
            ui.horizontal(|ui| {
                ui.label("endpoint");
                ui.text_edit_singleline(state.ollama_endpoint_mut());
                ui.label("model");
                ui.text_edit_singleline(state.ollama_model_mut());
            });
            if ui.button("Ollama に送信").clicked() {
                Self::store_string_result(state.run_ollama(), last_error);
            }
        });
    }

    fn store_result(
        result: Result<(), katana_chat_ui::ChatSessionError>,
        last_error: &mut Option<String>,
    ) {
        match result {
            Ok(()) => *last_error = None,
            Err(error) => *last_error = Some(error.to_string()),
        }
    }

    fn store_string_result(result: Result<(), String>, last_error: &mut Option<String>) {
        match result {
            Ok(()) => *last_error = None,
            Err(error) => *last_error = Some(error),
        }
    }
}
