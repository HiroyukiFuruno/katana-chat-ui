use crate::{
    args::StorybookArgs,
    scenario::{STORYBOOK_HISTORY_ID, provider_label, seeded_session, vendor_state},
};
use eframe::egui;
use katana_chat_ui::{
    Attachment, ChatSession, ChatUiHistorySessionSurface, ChatUiSurface, CommandLaunchKind,
    HostActionKind,
};
use katana_chat_ui_egui::{EguiChatController, EguiChatView};

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 900.0;
const STORYBOOK_ATTACHMENT: &str = "storybook.md";

pub(crate) fn run(args: StorybookArgs) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_active(!args.smoke),
        ..Default::default()
    };
    eframe::run_native(
        "KCU Storybook",
        options,
        Box::new(move |_| {
            let initial = seeded_session().map_err(std::io::Error::other)?;
            Ok(Box::new(StorybookApp::new(initial, args.smoke)))
        }),
    )
}

struct StorybookApp {
    session: ChatSession,
    view: EguiChatView,
    history_visible: bool,
    smoke_frames: Option<u8>,
    error: Option<String>,
}

impl StorybookApp {
    fn new(session: ChatSession, smoke: bool) -> Self {
        Self {
            session,
            view: EguiChatView::default(),
            history_visible: false,
            smoke_frames: smoke.then_some(0),
            error: None,
        }
    }

    fn surface(&self) -> ChatUiSurface {
        let mut surface = ChatUiSurface::from_render_model(&self.session.render_model());
        surface.history_panel.visible = self.history_visible;
        if self.history_visible {
            surface.history_panel.sessions = vec![ChatUiHistorySessionSurface {
                session_id: STORYBOOK_HISTORY_ID.to_string(),
                title: "KCU Storybook reference".to_string(),
                provider_id: surface.vendor_bar.active_vendor_id.clone(),
                provider_label: surface.vendor_bar.active_vendor_label.clone(),
                updated_at_label: "fixture".to_string(),
                preview: "KUC 共通 runtime の基準会話".to_string(),
            }];
        }
        surface
    }

    fn reset(&mut self) -> Result<(), String> {
        self.session = seeded_session()?;
        self.history_visible = false;
        Ok(())
    }

    fn submit_fixture_reply(&mut self) -> Result<(), String> {
        self.session
            .submit_draft()
            .map_err(|error| error.to_string())?;
        self.session
            .start_assistant_stream("Storybook の固定応答です。")
            .map_err(|error| error.to_string())?;
        self.session
            .append_assistant_chunk(
                "\n\n実 provider へ送信せず、KUC の入力・送信・表示経路だけを確認できます。",
            )
            .map_err(|error| error.to_string())?;
        self.session
            .finish_assistant_message()
            .map_err(|error| error.to_string())
    }

    fn select_control_value(&mut self, key: String, value: String) -> Result<(), String> {
        let mut state = self.session.vendor_ui_state().clone();
        let selected = match key.as_str() {
            "model" => (&state.model_options, &mut state.selected_model),
            "mode" => (&state.mode_options, &mut state.selected_mode),
            "thinking" => (&state.thinking_options, &mut state.selected_thinking),
            "permission" => (&state.permission_options, &mut state.selected_permission),
            _ => return Err(format!("unsupported Storybook control: {key}")),
        };
        if !selected.0.iter().any(|option| option == &value) {
            return Err(format!("unavailable Storybook {key} value: {value}"));
        }
        *selected.1 = Some(value);
        self.session.set_vendor_ui_state(state);
        Ok(())
    }

    fn advance_smoke(&mut self, context: &egui::Context) {
        let Some(frames) = self.smoke_frames.as_mut() else {
            return;
        };
        *frames += 1;
        if *frames >= 2 {
            context.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            context.request_repaint();
        }
    }
}

impl eframe::App for StorybookApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let surface = self.surface();
        let mut view = std::mem::take(&mut self.view);
        self.error = view.render(ui, &surface, self).err();
        self.view = view;
        if let Some(error) = &self.error {
            ui.label(error);
        }
        self.advance_smoke(ui.ctx());
    }
}

impl EguiChatController for StorybookApp {
    fn composer_text_mut(&mut self) -> &mut String {
        &mut self.session.draft_mut().text
    }

    fn attach(&mut self) {
        self.session.draft_mut().add_attachment(Attachment::text(
            STORYBOOK_ATTACHMENT,
            "# Storybook attachment\n\nprovider-free fixture",
        ));
    }

    fn submit(&mut self) -> Result<(), String> {
        self.submit_fixture_reply()
    }

    fn stop(&mut self) -> Result<(), String> {
        self.session
            .finish_assistant_message()
            .map_err(|error| error.to_string())
    }

    fn new_chat(&mut self) {
        self.error = self.reset().err();
    }

    fn history(&mut self) {
        self.history_visible = !self.history_visible;
    }

    fn select_vendor(&mut self, vendor_id: String) {
        self.error = (|| {
            self.session
                .set_provider_configured(provider_label(&vendor_id)?);
            self.session.set_vendor_ui_state(vendor_state(&vendor_id)?);
            Ok(())
        })()
        .err();
    }

    fn select_control(&mut self, key: String, value: String) {
        self.error = self.select_control_value(key, value).err();
    }

    fn remove_attachment(&mut self, index: usize) -> Result<(), String> {
        self.session
            .draft_mut()
            .remove_attachment(index)
            .map(|_| ())
            .ok_or_else(|| format!("missing Storybook attachment: {index}"))
    }

    fn select_history(&mut self, session_id: String) -> Result<(), String> {
        if session_id != STORYBOOK_HISTORY_ID {
            return Err(format!("unknown Storybook history: {session_id}"));
        }
        self.reset()
    }

    fn select_slash_command(
        &mut self,
        entry_id: String,
        _: CommandLaunchKind,
    ) -> Result<(), String> {
        Err(format!(
            "Storybook does not define slash command: {entry_id}"
        ))
    }

    fn output(&mut self, output_id: u64, _: HostActionKind) -> Result<(), String> {
        Err(format!(
            "Storybook does not define output action: {output_id}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::StorybookApp;
    use crate::scenario::{STORYBOOK_HISTORY_ID, seeded_session};
    use katana_chat_ui::{MessageRole, MessageStatus};
    use katana_chat_ui_egui::EguiChatController;

    #[test]
    fn submit_clears_draft_and_adds_completed_fixture_response() -> Result<(), String> {
        let mut app = StorybookApp::new(seeded_session()?, false);
        app.composer_text_mut().push_str(" 日本語入力");
        app.submit()?;

        let surface = app.surface();
        let latest = surface
            .messages
            .last()
            .ok_or("Storybook fixture response is missing")?;
        if !surface.composer.text.is_empty()
            || latest.role != MessageRole::Assistant
            || latest.status != MessageStatus::Complete
        {
            return Err("Storybook submit contract changed".to_string());
        }
        Ok(())
    }

    #[test]
    fn vendor_selection_includes_local_katanagent() -> Result<(), String> {
        let mut app = StorybookApp::new(seeded_session()?, false);
        app.select_vendor("katanagent".to_string());
        if app.surface().vendor_bar.active_vendor_id == "katanagent" {
            Ok(())
        } else {
            Err("KatanAgent is not selectable in Storybook".to_string())
        }
    }

    #[test]
    fn model_selection_updates_the_rendered_control() -> Result<(), String> {
        let mut app = StorybookApp::new(seeded_session()?, false);
        app.select_vendor("codex-cli".to_string());
        app.select_control("model".to_string(), "gpt-5.5".to_string());

        let surface = app.surface();
        let control = surface
            .vendor_bar
            .controls
            .iter()
            .find(|control| control.key == "model")
            .ok_or("Storybook model control is missing")?;
        assert_eq!(control.value, "gpt-5.5");
        Ok(())
    }

    #[test]
    fn selecting_storybook_history_resets_the_fixture_and_closes_history() -> Result<(), String> {
        let mut app = StorybookApp::new(seeded_session()?, false);
        app.history();
        assert!(app.surface().history_panel.visible);

        app.select_history(STORYBOOK_HISTORY_ID.to_string())?;

        let surface = app.surface();
        assert!(!surface.history_panel.visible);
        assert!(surface.composer.text.is_empty());
        assert_eq!(surface.messages.len(), 2);
        Ok(())
    }
}
