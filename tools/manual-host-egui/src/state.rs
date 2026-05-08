use crate::font::ManualFontStatus;
use katana_chat_ui::{
    AccountUsageSnapshot, Attachment, ChatSession, ChatSessionError, ChatUiSurface,
    ContextUsageSnapshot, FileResource,
};
use provider_config::{configure_manual_provider, manual_vendor_state, provider_label};

mod output_ops;
mod provider_config;

#[derive(Debug)]
pub struct ManualHostState {
    session: ChatSession,
    composer_text: String,
    last_action: String,
}

impl ManualHostState {
    pub fn new(_font_status: ManualFontStatus) -> Self {
        let mut session = ChatSession::new();
        configure_manual_provider(&mut session);
        session.set_context_usage(ContextUsageSnapshot::new(0, 200_000));
        session.set_account_usage(AccountUsageSnapshot::unavailable(
            "手動確認環境では account usage を取得しません",
        ));
        let mut state = Self {
            session,
            composer_text: String::new(),
            last_action: "起動しました".to_string(),
        };
        state.session.set_title("katana-chat-ui".to_string());
        state
    }

    pub fn render_model(&self) -> katana_chat_ui::ChatRenderModel {
        self.session.render_model()
    }

    pub fn surface(&self) -> ChatUiSurface {
        ChatUiSurface::from_render_model(&self.render_model())
    }

    pub fn composer_text_mut(&mut self) -> &mut String {
        &mut self.composer_text
    }

    pub fn add_sample_attachment(&mut self) {
        self.add_file_attachment("file:///tmp/sample.md", "# sample", 8);
        self.last_action = "サンプル添付を追加しました".to_string();
    }

    pub fn add_os_drop(&mut self, path: String) {
        let uri = format!("file://{path}");
        self.add_file_attachment(uri, "OS dropped file placeholder", 27);
        self.last_action = "OS の drop を追加しました".to_string();
    }

    pub fn submit(&mut self) -> Result<(), ChatSessionError> {
        self.submit_user_message()?;
        let assistant_id = self
            .session
            .start_assistant_stream("mock provider response")?;
        self.add_sample_outputs(assistant_id)?;
        self.composer_text.clear();
        self.last_action = "送信して streaming 中にしました".to_string();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), ChatSessionError> {
        self.session.finish_assistant_message()?;
        self.last_action = "停止して応答を完了扱いにしました".to_string();
        Ok(())
    }

    pub fn start_new_chat(&mut self) {
        let mut session = ChatSession::new();
        configure_manual_provider(&mut session);
        session.set_context_usage(ContextUsageSnapshot::new(0, 200_000));
        self.session = session;
        self.composer_text.clear();
        self.last_action = "新しい会話を開始しました".to_string();
    }

    pub fn open_history(&mut self) {
        self.last_action = "履歴を開きました".to_string();
    }

    pub fn open_settings(&mut self) {
        self.session.toggle_settings();
        self.last_action = "設定を切り替えました".to_string();
    }

    pub fn select_vendor(&mut self, vendor_id: String) {
        self.session
            .set_provider_configured(provider_label(&vendor_id).to_string());
        self.session
            .set_vendor_ui_state(manual_vendor_state(&vendor_id));
        self.last_action = format!("provider を {vendor_id} に変更しました");
    }

    pub fn select_control(&mut self, key: String, value: String) {
        let mut state = self.session.vendor_ui_state().clone();
        match key.as_str() {
            "model" => state.selected_model = Some(value.clone()),
            "thinking" => state.selected_thinking = Some(value.clone()),
            "permission" => state.selected_permission = Some(value.clone()),
            _ => {
                self.last_action = format!("未対応の control です: {key}");
                return;
            }
        }
        self.session.set_vendor_ui_state(state);
        self.last_action = format!("{key} を {value} に変更しました");
    }

    fn add_file_attachment(&mut self, uri: impl Into<String>, text: impl Into<String>, size: u64) {
        self.session
            .draft_mut()
            .add_attachment(Attachment::file(FileResource::new(
                uri,
                "text/markdown",
                text,
                size,
            )));
    }

    pub(crate) fn submit_user_message(&mut self) -> Result<u64, ChatSessionError> {
        self.session
            .draft_mut()
            .set_text(self.composer_text.clone());
        self.session.submit_draft()
    }
}

impl Default for ManualHostState {
    fn default() -> Self {
        Self::new(ManualFontStatus::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::ManualHostState;

    #[test]
    fn submit_creates_user_and_streaming_assistant_messages() -> Result<(), String> {
        let mut state = ManualHostState::default();
        state
            .composer_text_mut()
            .push_str("egui host submit contract");
        state.submit().map_err(|it| it.to_string())?;

        let model = state.render_model();

        assert!(model.messages.len() >= 2);
        assert!(model.input.can_cancel);
        Ok(())
    }

    #[test]
    fn submit_rejects_empty_draft() {
        let mut state = ManualHostState::default();

        let result = state.submit();

        assert!(result.is_err());
    }
}
