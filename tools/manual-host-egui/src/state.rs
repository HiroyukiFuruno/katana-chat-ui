use crate::font::ManualFontStatus;
use katana_chat_ui::{
    AccountUsageSnapshot, Attachment, ChatSession, ChatSessionError, ContextUsageSnapshot,
    FileResource,
};

mod ollama_ops;
mod output_ops;

#[derive(Debug)]
pub struct ManualHostState {
    session: ChatSession,
    composer_text: String,
    path_drop_text: String,
    ollama_endpoint: String,
    ollama_model: String,
    font_status: ManualFontStatus,
    last_action: String,
}

impl ManualHostState {
    pub fn new(font_status: ManualFontStatus) -> Self {
        let mut session = ChatSession::new();
        session.set_provider_configured("手動確認用 provider（manual provider）");
        session.set_context_usage(ContextUsageSnapshot::new(75_000, 100_000));
        session.set_account_usage(AccountUsageSnapshot::unavailable(
            "手動確認環境では account usage を取得しません",
        ));
        let mut state = Self {
            session,
            composer_text: "この UI を手で確認します".to_string(),
            path_drop_text: "/tmp/kcu-manual.md".to_string(),
            ollama_endpoint: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
            font_status,
            last_action: "起動しました".to_string(),
        };
        state.seed_demo_content();
        state
    }

    pub fn render_model(&self) -> katana_chat_ui::ChatRenderModel {
        self.session.render_model()
    }

    pub fn composer_text_mut(&mut self) -> &mut String {
        &mut self.composer_text
    }

    pub fn path_drop_text_mut(&mut self) -> &mut String {
        &mut self.path_drop_text
    }

    pub fn ollama_endpoint_mut(&mut self) -> &mut String {
        &mut self.ollama_endpoint
    }

    pub fn ollama_model_mut(&mut self) -> &mut String {
        &mut self.ollama_model
    }

    pub fn last_action(&self) -> &str {
        &self.last_action
    }

    pub fn font_status_label(&self) -> String {
        self.font_status.label()
    }

    pub fn add_sample_attachment(&mut self) {
        self.add_file_attachment("file:///tmp/sample.md", "# sample", 8);
        self.last_action = "サンプル添付を追加しました".to_string();
    }

    pub fn add_path_drop(&mut self) {
        let uri = self.path_drop_uri();
        self.add_file_attachment(uri, "path drop placeholder", 21);
        self.last_action = "path drop を追加しました".to_string();
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
            .start_assistant_stream("手動確認用の応答を生成中です")?;
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

    fn path_drop_uri(&self) -> String {
        if self.path_drop_text.starts_with("file://") {
            return self.path_drop_text.clone();
        }
        format!("file://{}", self.path_drop_text)
    }

    pub(crate) fn submit_user_message(&mut self) -> Result<u64, ChatSessionError> {
        self.session
            .draft_mut()
            .set_text(self.composer_text.clone());
        self.session.submit_draft()
    }

    fn seed_demo_content(&mut self) {
        match self.add_initial_assistant_message() {
            Ok(()) => self.last_action = "起動時サンプルを表示しました".to_string(),
            Err(error) => self.last_action = format!("起動時サンプル表示に失敗: {error}"),
        }
    }

    fn add_initial_assistant_message(&mut self) -> Result<(), ChatSessionError> {
        let assistant_id = self.session.start_assistant_stream(
            "これは取り込み側hostの手動確認画面です。入力、添付、output 操作、Ollama 送信を確認します。",
        )?;
        self.session.finish_assistant_message()?;
        self.add_sample_outputs(assistant_id)?;
        Ok(())
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
        state.submit().map_err(|it| it.to_string())?;

        let model = state.render_model();

        assert!(model.messages.len() >= 3);
        assert!(model.input.can_cancel);
        Ok(())
    }
}
