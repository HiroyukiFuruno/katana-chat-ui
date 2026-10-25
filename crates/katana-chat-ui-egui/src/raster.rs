mod events;
mod presentation;

use crate::EguiChatController;
use eframe::egui;
use katana_chat_ui::ChatUiSurface;
use katana_chat_ui::kuc::{
    ChatUiRuntime, ChatUiRuntimeClipboard, ChatUiRuntimeEffect, ChatUiRuntimeFrame,
    ChatUiRuntimeViewport,
};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct EguiKucView {
    runtime: Option<ChatUiRuntime>,
    texture: Option<egui::TextureHandle>,
    ime_trace_seq: u64,
}

impl EguiKucView {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) -> Result<(), String> {
        let mut runtime = match self.runtime.take() {
            Some(runtime) => runtime,
            None => ChatUiRuntime::light(surface).map_err(|error| error.to_string())?,
        };
        let result = self.render_runtime(ui, surface, controller, &mut runtime);
        self.runtime = Some(runtime);
        result
    }

    fn render_runtime(
        &mut self,
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
        runtime: &mut ChatUiRuntime,
    ) -> Result<(), String> {
        let size = ui.available_size().max(egui::vec2(1.0, 1.0));
        let max_side = ui.input(|input| input.max_texture_side) as f32;
        if size.max_elem() * ui.ctx().pixels_per_point() > max_side {
            return Err("KUC viewport exceeds the host texture limit".into());
        }
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        let viewport = ChatUiRuntimeViewport::new(
            size.x.ceil() as u32,
            size.y.ceil() as u32,
            ui.ctx().pixels_per_point(),
        );
        let frame = process_input(ui, &response, surface, controller, runtime, viewport)?;
        self.trace_ime_state(frame.composer.text_surface.preedit.is_some());
        presentation::paint(ui, rect, &frame, &mut self.texture);
        if response.has_focus() {
            presentation::ime(ui, rect, &frame);
        }
        Ok(())
    }

    fn trace_ime_state(&mut self, marked: bool) {
        if env::var("KCU_IME_STATE_TRACE").as_deref() != Ok("1") {
            return;
        }
        let source = "egui-kuc-runtime-frame";
        self.ime_trace_seq = self.ime_trace_seq.saturating_add(1);
        eprintln!(
            "{}",
            ime_state_output(source, self.ime_trace_seq, SystemTime::now(), marked)
        );
    }
}

fn ime_state_record(
    source: &str,
    sequence: u64,
    observed_at: SystemTime,
    marked: bool,
) -> Result<String, &'static str> {
    let seconds = observed_at
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "before-unix-epoch")?
        .as_secs_f64();
    Ok(format!(
        "KCU_IME_STATE {{\"kind\":\"ime-state\",\"source\":\"{source}\",\"seq\":{sequence},\"observed_at\":{seconds:.6},\"marked\":{marked}}}"
    ))
}

fn ime_state_output(source: &str, sequence: u64, observed_at: SystemTime, marked: bool) -> String {
    match ime_state_record(source, sequence, observed_at, marked) {
        Ok(record) => record,
        Err(error) => format!("KCU_IME_STATE_ERROR source={source} clock={error}"),
    }
}

fn process_input(
    ui: &mut egui::Ui,
    response: &egui::Response,
    surface: &ChatUiSurface,
    controller: &mut impl EguiChatController,
    runtime: &mut ChatUiRuntime,
    viewport: ChatUiRuntimeViewport,
) -> Result<ChatUiRuntimeFrame, String> {
    let mut current = surface.clone();
    current.composer.text = controller.composer_text_mut().clone();
    let mut frame = runtime
        .render(&current, viewport)
        .map_err(|error| error.to_string())?;
    for input in events::collect(ui, response) {
        let effects = runtime
            .dispatch(&frame, input)
            .map_err(|error| error.to_string())?;
        apply_effects(ui, response, controller, effects)?;
        current.composer.text = controller.composer_text_mut().clone();
        frame = runtime
            .render(&current, viewport)
            .map_err(|error| error.to_string())?;
    }
    Ok(frame)
}

fn apply_effects(
    ui: &mut egui::Ui,
    response: &egui::Response,
    controller: &mut impl EguiChatController,
    effects: Vec<ChatUiRuntimeEffect>,
) -> Result<(), String> {
    for effect in effects {
        match effect {
            ChatUiRuntimeEffect::Action(action) => controller.handle_action(action)?,
            ChatUiRuntimeEffect::DraftChanged(text) => *controller.composer_text_mut() = text,
            ChatUiRuntimeEffect::Clipboard(ChatUiRuntimeClipboard::Write(text)) => {
                ui.ctx().copy_text(text);
            }
            ChatUiRuntimeEffect::Clipboard(ChatUiRuntimeClipboard::Read) => {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
            }
            ChatUiRuntimeEffect::FocusComposer => response.request_focus(),
            ChatUiRuntimeEffect::Repaint => ui.ctx().request_repaint(),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ime_trace_tests {
    use super::{ime_state_output, ime_state_record};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn formats_typed_state_record() -> Result<(), &'static str> {
        let record = ime_state_record(
            "egui-kuc-runtime-frame",
            7,
            UNIX_EPOCH + Duration::from_secs_f64(12.5),
            true,
        )?;
        assert!(record.contains("\"seq\":7"));
        assert!(record.contains("\"observed_at\":12.500000"));
        assert!(record.ends_with("\"marked\":true}"));
        Ok(())
    }

    #[test]
    fn rejects_pre_epoch_clock_without_state_record() {
        let result = ime_state_record(
            "egui-kuc-runtime-frame",
            8,
            UNIX_EPOCH - Duration::from_secs(1),
            false,
        );
        assert_eq!(result, Err("before-unix-epoch"));
        assert_eq!(
            ime_state_output(
                "egui-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                false,
            ),
            "KCU_IME_STATE_ERROR source=egui-kuc-runtime-frame clock=before-unix-epoch"
        );
    }
}
