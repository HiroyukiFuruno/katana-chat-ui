use std::{
    fs,
    path::{Path, PathBuf},
};

use katana_chat_ui::{
    Attachment, ChatOutputKind, ChatSession, ChatSessionError, ChatTextKey, ChatUiConfig,
    ChatUiOptions, ChatUiSurface, DiffCandidateOutput, FileResource, TextCatalog, ThinkingLog,
    VendorUiState,
};

use crate::provider::{
    ManualProviderEvent, ManualProviderJob, ManualProviderRegistry, current_working_dir,
};

pub(crate) type ManualFloemError = ChatSessionError;

const MAX_ATTACHMENT_PROMPT_CHARS: usize = 12_000;
const ATTACHMENT_TRUNCATED_NOTICE: &str = "\n\n[attachment truncated for local provider request]";

#[derive(Clone)]
pub(crate) struct ManualFloemState {
    session: ChatSession,
    providers: ManualProviderRegistry,
    active_assistant_message_id: Option<u64>,
    title_updated: bool,
    pub(crate) last_event: String,
}

impl ManualFloemState {
    pub(crate) fn new() -> Result<Self, ManualFloemError> {
        let mut session = ChatSession::with_config(manual_host_config());
        let providers = ManualProviderRegistry::discover();
        let last_event = configure_provider_state(&mut session, &providers);
        session.set_text_catalog(
            TextCatalog::english()
                .with_text(ChatTextKey::SendButton, "Run")
                .with_text(ChatTextKey::SettingsButton, "Output"),
        );
        Ok(Self {
            session,
            providers,
            active_assistant_message_id: None,
            title_updated: false,
            last_event,
        })
    }

    #[cfg(test)]
    pub(crate) fn attach_sample(&mut self) {
        self.session
            .draft_mut()
            .add_attachment(Attachment::text("sample.md", "# sample"));
        self.last_event = "添付を追加しました".to_string();
    }

    pub(crate) fn attach_selected_paths(&mut self, paths: Vec<PathBuf>) {
        match self.attach_paths(paths) {
            Ok(count) => self.last_event = format!("添付を追加しました: {count} 件"),
            Err(error) => self.last_event = format!("添付失敗: {error}"),
        }
    }

    pub(crate) fn attach_cancelled(&mut self) {
        self.last_event = "添付をキャンセルしました".to_string();
    }

    pub(crate) fn remove_attachment(&mut self, index: usize) {
        match self.session.draft_mut().remove_attachment(index) {
            Some(_) => self.last_event = "添付を削除しました".to_string(),
            None => self.last_event = format!("添付削除失敗: index {index}"),
        }
    }

    pub(crate) fn refresh_ollama_models(&mut self) {
        self.refresh_provider_registry(ManualProviderRegistry::discover());
    }

    pub(crate) fn add_harness_sample_history(&mut self) {
        match self.try_add_harness_sample_history() {
            Ok(()) => self.last_event = "harness 履歴サンプルを追加しました".to_string(),
            Err(error) => self.last_event = format!("harness サンプル追加失敗: {error}"),
        }
    }

    fn refresh_provider_registry(&mut self, providers: ManualProviderRegistry) {
        self.providers = providers;
        let active_vendor_id = self.session.vendor_ui_state().active_vendor_id.clone();
        let state = self
            .providers
            .vendor_state_for(&active_vendor_id)
            .or_else(|| self.providers.first_vendor_state());
        match state {
            Some(state) => {
                self.session
                    .set_provider_configured(state.active_vendor_id.clone());
                self.session.set_vendor_ui_state(state);
                self.last_event = "利用可能 provider を再検出しました".to_string();
            }
            None => {
                self.session
                    .set_provider_missing("利用可能な ACP provider がありません");
                self.session
                    .set_vendor_ui_state(VendorUiState::without_vendor());
                self.last_event =
                    "provider 再検出失敗: 利用可能な provider がありません".to_string();
            }
        }
    }

    pub(crate) fn select_vendor(&mut self, vendor_id: String) {
        let Some(state) = self.providers.vendor_state_for(&vendor_id) else {
            self.last_event = format!("利用できない provider です: {vendor_id}");
            return;
        };
        self.session.set_provider_configured(vendor_id.clone());
        self.session.set_vendor_ui_state(state);
        self.last_event = format!("provider を {vendor_id} に変更しました");
    }

    pub(crate) fn select_control(&mut self, key: String, value: String) {
        let mut state = self.session.vendor_ui_state().clone();
        match key.as_str() {
            "endpoint" => state.endpoint = Some(value.clone()),
            "model" => state.selected_model = Some(value.clone()),
            "mode" => state.selected_mode = Some(value.clone()),
            "thinking" => state.selected_thinking = Some(value.clone()),
            "permission" => state.selected_permission = Some(value.clone()),
            _ => {
                self.last_event = format!("未対応の control です: {key}");
                return;
            }
        }
        self.session.set_vendor_ui_state(state);
        self.last_event = format!("{key} を {value} に変更しました");
    }

    pub(crate) fn start_submit(&mut self, text: String) -> Option<ManualProviderJob> {
        match self.try_start_submit(text) {
            Ok(job) => {
                self.last_event = format!("送信しました: {} 応答待ち", job.vendor_id);
                Some(job)
            }
            Err(error) => {
                self.last_event = format!("送信失敗: {error}");
                None
            }
        }
    }

    pub(crate) fn apply_provider_event(&mut self, event: ManualProviderEvent) {
        match event {
            ManualProviderEvent::Chunk {
                assistant_message_id,
                vendor_id,
                content,
            } => self.apply_provider_chunk(assistant_message_id, vendor_id, content),
            ManualProviderEvent::ThinkingChunk {
                assistant_message_id,
                vendor_id,
                content,
            } => self.apply_provider_thinking(assistant_message_id, vendor_id, content),
            ManualProviderEvent::Finished {
                assistant_message_id,
                vendor_id,
            } => self.finish_provider_response(assistant_message_id, vendor_id),
            ManualProviderEvent::Failed {
                assistant_message_id,
                vendor_id,
                error,
            } => self.fail_provider_response(assistant_message_id, vendor_id, error),
        }
    }

    pub(crate) fn stop(&mut self) {
        match self.session.finish_assistant_message() {
            Ok(()) => {
                self.active_assistant_message_id = None;
                self.last_event = "停止しました".to_string();
            }
            Err(error) => self.last_event = format!("停止失敗: {error}"),
        }
    }

    pub(crate) fn surface(&self) -> ChatUiSurface {
        ChatUiSurface::from_render_model(&self.session.render_model())
    }

    pub(crate) fn vendor_summary(&self) -> String {
        let state = self.session.vendor_ui_state();
        [
            format!("provider: {}", state.active_vendor_id),
            format!("endpoint: {}", endpoint_label(state)),
            format!("model: {}", model_label(state)),
            format!("thinking: {}", thinking_label(state)),
        ]
        .join("\n")
    }

    fn try_start_submit(&mut self, text: String) -> Result<ManualProviderJob, String> {
        if self.active_assistant_message_id.is_some() {
            return Err("provider response is already running".to_string());
        }
        let vendor_state = self.session.vendor_ui_state().clone();
        if !self.providers.contains(&vendor_state.active_vendor_id) {
            return Err(format!(
                "provider is not available: {}",
                vendor_state.active_vendor_id
            ));
        }
        self.session.draft_mut().set_text(text);
        let prompt = Self::provider_prompt(self.session.draft());
        self.session
            .submit_draft()
            .map_err(|error| error.to_string())?;
        let thinking_log = Self::thinking_log_start(&vendor_state);
        let assistant_id = match thinking_log {
            Some(thinking) => self
                .session
                .start_assistant_stream_with_thinking(String::new(), thinking)
                .map_err(|error| error.to_string())?,
            None => self
                .session
                .start_assistant_stream(String::new())
                .map_err(|error| error.to_string())?,
        };
        self.active_assistant_message_id = Some(assistant_id);
        Ok(ManualProviderJob {
            assistant_message_id: assistant_id,
            vendor_id: vendor_state.active_vendor_id,
            endpoint: vendor_state.endpoint,
            model: vendor_state.selected_model,
            thinking: vendor_state.selected_thinking,
            permission: vendor_state.selected_permission,
            prompt,
            cwd: current_working_dir()?,
        })
    }

    fn apply_provider_chunk(&mut self, assistant_id: u64, vendor_id: String, content: String) {
        if self.reject_inactive(assistant_id, &vendor_id) {
            return;
        }
        self.update_title_from_response(&content);
        match self.session.append_assistant_chunk(content) {
            Ok(()) => self.last_event = format!("{vendor_id} 応答受信中"),
            Err(error) => self.last_event = format!("{vendor_id} 応答反映失敗: {error}"),
        }
    }

    fn apply_provider_thinking(&mut self, assistant_id: u64, vendor_id: String, content: String) {
        if self.reject_inactive(assistant_id, &vendor_id) {
            return;
        }
        match self.session.append_assistant_thinking("Thinking", content) {
            Ok(()) => self.last_event = format!("{vendor_id} 考慮ログ受信中"),
            Err(error) => self.last_event = format!("{vendor_id} 考慮ログ反映失敗: {error}"),
        }
    }

    fn finish_provider_response(&mut self, assistant_id: u64, vendor_id: String) {
        if self.reject_inactive(assistant_id, &vendor_id) {
            return;
        }
        match self.finish_active_response(assistant_id) {
            Ok(()) => {
                self.active_assistant_message_id = None;
                self.last_event = format!("{vendor_id} 応答を受信しました");
            }
            Err(error) => {
                self.active_assistant_message_id = None;
                self.last_event = format!("{vendor_id} 応答反映失敗: {error}");
            }
        }
    }

    fn fail_provider_response(&mut self, assistant_id: u64, vendor_id: String, error: String) {
        if self.reject_inactive(assistant_id, &vendor_id) {
            return;
        }
        if let Err(error) = self
            .session
            .fail_assistant_message(format!("応答失敗: {error}"))
        {
            self.last_event = format!("{vendor_id} 応答失敗反映失敗: {error}");
            return;
        }
        self.active_assistant_message_id = None;
        self.last_event = format!("{vendor_id} 応答失敗: {error}");
    }

    fn finish_active_response(&mut self, _assistant_id: u64) -> Result<(), String> {
        self.session
            .finish_assistant_message()
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn reject_inactive(&mut self, assistant_id: u64, vendor_id: &str) -> bool {
        if self.active_assistant_message_id == Some(assistant_id) {
            return false;
        }
        self.last_event = format!("{vendor_id} 応答を破棄しました: inactive request");
        true
    }

    fn thinking_log_start(state: &VendorUiState) -> Option<ThinkingLog> {
        if thinking_is_disabled(state) {
            return None;
        }
        Some(ThinkingLog::running("Thinking", Vec::new()))
    }

    fn sample_output() -> ChatOutputKind {
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/floem-generated.md",
            "--- a/tmp/floem-generated.md\n+++ b/tmp/floem-generated.md\n@@ -1 +1 @@\n-before\n+after\n",
        ))
    }

    fn try_add_harness_sample_history(&mut self) -> Result<(), String> {
        self.session.draft_mut().set_text("履歴表示サンプル");
        self.session
            .submit_draft()
            .map_err(|error| error.to_string())?;
        let assistant_id = self
            .session
            .start_assistant_stream("harness 専用の履歴表示サンプルです")
            .map_err(|error| error.to_string())?;
        self.session
            .finish_assistant_message()
            .map_err(|error| error.to_string())?;
        self.session
            .add_output(assistant_id, Self::sample_output())
            .map(|_| ())
            .map_err(|error| format!("output 追加失敗: {error}"))
    }

    fn attach_paths(&mut self, paths: Vec<PathBuf>) -> Result<usize, String> {
        if paths.is_empty() {
            return Err("selected file list is empty".to_string());
        }
        let attachments = paths
            .iter()
            .map(|path| Self::attachment_from_path(path))
            .collect::<Result<Vec<_>, _>>()?;
        let count = attachments.len();
        for attachment in attachments {
            self.session.draft_mut().add_attachment(attachment);
        }
        Ok(count)
    }

    fn attachment_from_path(path: &Path) -> Result<Attachment, String> {
        let text =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let metadata =
            fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(Attachment::file(FileResource::new(
            Self::file_uri(path)?,
            Self::mime_type(path),
            text,
            metadata.len(),
        )))
    }

    fn file_uri(path: &Path) -> Result<String, String> {
        let Some(path) = path.to_str() else {
            return Err(format!("path is not UTF-8: {path:?}"));
        };
        Ok(format!("file://{path}"))
    }

    fn mime_type(path: &Path) -> &'static str {
        match path.extension().and_then(|it| it.to_str()) {
            Some("md" | "markdown") => "text/markdown",
            Some("json") => "application/json",
            Some("rs") => "text/rust",
            Some("toml") => "application/toml",
            Some("yaml" | "yml") => "application/yaml",
            Some("txt") => "text/plain",
            _ => "text/plain",
        }
    }

    fn provider_prompt(draft: &katana_chat_ui::ChatInputDraft) -> String {
        let mut prompt = draft.text.clone();
        for attachment in &draft.attachments {
            prompt.push_str("\n\n");
            prompt.push_str(&Self::attachment_prompt(attachment));
        }
        prompt
    }

    fn attachment_prompt(attachment: &Attachment) -> String {
        match attachment {
            Attachment::FileResource(file) => {
                format!(
                    "Attached file: {}\n```\n{}\n```",
                    file.uri,
                    Self::prompt_attachment_text(&file.text)
                )
            }
            Attachment::Text(text) => {
                format!(
                    "Attached text: {}\n{}",
                    text.label,
                    Self::prompt_attachment_text(&text.text)
                )
            }
            Attachment::ImageResource(image) => {
                format!("Attached image: {} ({})", image.data_ref, image.mime_type)
            }
            Attachment::Unsupported(unsupported) => {
                format!("Unsupported attachment: {}", unsupported.label)
            }
        }
    }

    fn prompt_attachment_text(text: &str) -> String {
        let mut chars = text.chars();
        let limited: String = chars.by_ref().take(MAX_ATTACHMENT_PROMPT_CHARS).collect();
        if chars.next().is_none() {
            return limited;
        }
        format!("{limited}{ATTACHMENT_TRUNCATED_NOTICE}")
    }

    fn update_title_from_response(&mut self, content: &str) {
        if self.title_updated {
            return;
        }
        let title = Self::title_from_response(content);
        if title.is_empty() {
            return;
        }
        self.session.set_title(title);
        self.title_updated = true;
    }

    fn title_from_response(content: &str) -> String {
        content
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(str::trim)
            .unwrap_or_default()
            .chars()
            .take(42)
            .collect()
    }

    #[cfg(test)]
    fn new_for_test() -> Result<Self, ManualFloemError> {
        Self::new_for_test_with_providers(ManualProviderRegistry::for_test_ollama())
    }

    #[cfg(test)]
    fn new_for_test_with_providers(
        providers: ManualProviderRegistry,
    ) -> Result<Self, ManualFloemError> {
        let mut session = ChatSession::new();
        session.apply_config(manual_host_config());
        configure_provider_state(&mut session, &providers);
        Ok(Self {
            session,
            providers,
            active_assistant_message_id: None,
            title_updated: false,
            last_event: "起動しました".to_string(),
        })
    }

    #[cfg(test)]
    fn attach_selected_file_for_test(&mut self, path: PathBuf) {
        self.attach_selected_paths(vec![path]);
    }

    #[cfg(test)]
    fn refresh_with_providers_for_test(&mut self, providers: ManualProviderRegistry) {
        self.refresh_provider_registry(providers);
    }
}

fn manual_host_config() -> ChatUiConfig {
    ChatUiConfig::default().with_options(ChatUiOptions::default().with_debug(true))
}

fn configure_provider_state(
    session: &mut ChatSession,
    providers: &ManualProviderRegistry,
) -> String {
    match providers.first_vendor_state() {
        Some(state) => {
            session.set_provider_configured(state.active_vendor_id.clone());
            session.set_vendor_ui_state(state);
            "起動しました: 利用可能 provider を検出しました".to_string()
        }
        None => {
            session.set_provider_missing("利用可能な ACP provider がありません");
            session.set_vendor_ui_state(VendorUiState::without_vendor());
            "起動しました: 利用可能な provider がありません".to_string()
        }
    }
}

fn endpoint_label(state: &VendorUiState) -> String {
    match &state.endpoint {
        Some(endpoint) => endpoint.clone(),
        None => "Unavailable".to_string(),
    }
}

fn model_label(state: &VendorUiState) -> String {
    match &state.selected_model {
        Some(model) => model.clone(),
        None => "Unavailable".to_string(),
    }
}

fn thinking_label(state: &VendorUiState) -> String {
    match &state.selected_thinking {
        Some(thinking) => thinking.clone(),
        None => "Unavailable".to_string(),
    }
}

fn thinking_is_disabled(state: &VendorUiState) -> bool {
    matches!(
        state.selected_thinking.as_deref(),
        None | Some("false" | "default")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        ATTACHMENT_TRUNCATED_NOTICE, MAX_ATTACHMENT_PROMPT_CHARS, ManualFloemState,
        ManualProviderRegistry,
    };
    use crate::provider::ManualProviderEvent;
    use katana_chat_ui::Attachment;
    use std::{fs, path::PathBuf};

    #[test]
    fn initial_state_does_not_seed_manual_host_messages() -> Result<(), super::ManualFloemError> {
        let state = ManualFloemState::new_for_test()?;

        let surface = state.surface();
        assert!(surface.message_list.messages.is_empty());
        assert!(surface.output_handoff.outputs.is_empty());
        Ok(())
    }

    #[test]
    fn attach_sample_preserves_messages_and_draft_text() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let before_messages = state.surface().message_list.messages;
        state.session.draft_mut().set_text("こんにちは");

        state.attach_sample();

        let surface = state.surface();
        assert_eq!(surface.message_list.messages, before_messages);
        assert_eq!(surface.composer.text, "こんにちは");
        assert_eq!(surface.composer.attachments.len(), 1);
        Ok(())
    }

    #[test]
    fn harness_sample_history_is_explicit_not_startup_seed()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;

        state.add_harness_sample_history();

        let surface = state.surface();
        assert_eq!(surface.message_list.messages.len(), 2);
        assert_eq!(
            surface.message_list.messages[0].body,
            "履歴表示サンプル".to_string()
        );
        assert_eq!(
            state.last_event,
            "harness 履歴サンプルを追加しました".to_string()
        );
        Ok(())
    }

    #[test]
    fn submit_adds_user_turn_and_streaming_stop_state() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let before_count = state.surface().message_list.messages.len();

        let job = state.start_submit("送信確認".to_string());

        let surface = state.surface();
        assert_eq!(surface.message_list.messages.len(), before_count + 2);
        assert!(surface.composer.stop_enabled);
        assert!(job.is_some());
        assert_eq!(state.last_event, "送信しました: ollama 応答待ち");
        Ok(())
    }

    #[test]
    fn select_vendor_rejects_unavailable_provider() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;

        state.select_vendor("missing-provider".to_string());

        assert_eq!(
            state.session.vendor_ui_state().active_vendor_id,
            "ollama".to_string()
        );
        assert_eq!(
            state.last_event,
            "利用できない provider です: missing-provider"
        );
        Ok(())
    }

    #[test]
    fn empty_provider_registry_exposes_no_vendor_selector_options()
    -> Result<(), super::ManualFloemError> {
        let state = ManualFloemState::new_for_test_with_providers(
            ManualProviderRegistry::empty_for_test(),
        )?;

        let surface = state.surface();
        assert!(surface.vendor_bar.active_vendor_id.is_empty());
        assert!(surface.vendor_bar.active_vendor_label.is_empty());
        assert!(surface.vendor_bar.vendor_options.is_empty());
        assert!(!surface.composer.send_available);
        Ok(())
    }

    #[test]
    fn provider_events_finish_streaming_state() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let Some(job) = state.start_submit("送信確認".to_string()) else {
            panic!("job should start");
        };
        let before_messages = state.surface().message_list.messages;

        state.apply_provider_event(ManualProviderEvent::Chunk {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            content: "応答しました".to_string(),
        });
        state.apply_provider_event(ManualProviderEvent::Finished {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
        });

        let surface = state.surface();
        assert_eq!(surface.message_list.messages.len(), before_messages.len());
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .map(|it| it.outputs.is_empty()),
            Some(true)
        );
        assert!(!surface.composer.stop_enabled);
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .map(|it| it.body.clone()),
            Some("応答しました".to_string())
        );
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .and_then(|it| it.thinking.as_ref())
                .map(|it| it.completed),
            None
        );
        Ok(())
    }

    #[test]
    fn thinking_false_does_not_create_thinking_surface() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;

        state.start_submit("考慮ログなし".to_string());

        assert!(
            state
                .surface()
                .message_list
                .messages
                .last()
                .and_then(|it| it.thinking.as_ref())
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn provider_event_marks_error_state() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let Some(job) = state.start_submit("送信確認".to_string()) else {
            panic!("job should start");
        };

        state.apply_provider_event(ManualProviderEvent::Failed {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
            error: "connection failed".to_string(),
        });

        let surface = state.surface();
        assert!(!surface.composer.stop_enabled);
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .map(|it| &it.status),
            Some(&katana_chat_ui::MessageStatus::Error(
                "応答失敗: connection failed".to_string()
            ))
        );
        assert_eq!(state.last_event, "ollama 応答失敗: connection failed");
        Ok(())
    }

    #[test]
    fn submit_truncates_large_attachment_prompt() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let large_text = "あ".repeat(MAX_ATTACHMENT_PROMPT_CHARS + 1);
        state
            .session
            .draft_mut()
            .add_attachment(Attachment::text("large.md", large_text));

        let Some(job) = state.start_submit("添付確認".to_string()) else {
            panic!("job should start");
        };

        assert!(job.prompt.contains(ATTACHMENT_TRUNCATED_NOTICE));
        Ok(())
    }

    #[test]
    fn refresh_provider_registry_keeps_standard_send_icon()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let before = state.surface().composer.send.icon.svg;

        state.refresh_with_providers_for_test(ManualProviderRegistry::for_test_ollama());

        assert_eq!(state.surface().composer.send.icon.svg, before);
        assert!(!state.surface().composer.send.icon.svg.contains("send-alt"));
        Ok(())
    }

    #[test]
    fn stopped_provider_result_is_not_applied_to_next_message()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let Some(stopped_job) = state.start_submit("古い送信".to_string()) else {
            panic!("first job should start");
        };
        state.stop();
        let Some(active_job) = state.start_submit("新しい送信".to_string()) else {
            panic!("second job should start");
        };

        state.apply_provider_event(ManualProviderEvent::Chunk {
            assistant_message_id: stopped_job.assistant_message_id,
            vendor_id: stopped_job.vendor_id,
            content: "古い応答".to_string(),
        });

        let surface = state.surface();
        assert!(surface.composer.stop_enabled);
        assert_eq!(
            surface.message_list.messages.last().map(|it| it.id),
            Some(active_job.assistant_message_id)
        );
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .map(|it| it.body.as_str()),
            Some("")
        );
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .and_then(|it| it.thinking.as_ref())
                .map(|it| it.entries.iter().any(|entry| entry == "provider: ollama")),
            None
        );
        Ok(())
    }

    #[test]
    fn selected_file_can_be_attached_and_removed() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let path = temp_attachment_path("kcu-manual-attach.md");
        fs::write(&path, "# selected").expect("test attachment should be writable");

        state.attach_selected_file_for_test(path.clone());
        state.remove_attachment(0);

        let surface = state.surface();
        assert!(surface.composer.attachments.is_empty());
        assert_eq!(state.last_event, "添付を削除しました");
        fs::remove_file(path).expect("test attachment should be removable");
        Ok(())
    }

    #[test]
    fn active_provider_result_after_stopped_result_is_applied()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let Some(stopped_job) = state.start_submit("古い送信".to_string()) else {
            panic!("first job should start");
        };
        state.stop();
        let Some(active_job) = state.start_submit("新しい送信".to_string()) else {
            panic!("second job should start");
        };

        state.apply_provider_event(ManualProviderEvent::Chunk {
            assistant_message_id: stopped_job.assistant_message_id,
            vendor_id: stopped_job.vendor_id,
            content: "古い応答".to_string(),
        });
        state.apply_provider_event(ManualProviderEvent::Chunk {
            assistant_message_id: active_job.assistant_message_id,
            vendor_id: active_job.vendor_id.clone(),
            content: "新しい応答".to_string(),
        });
        state.apply_provider_event(ManualProviderEvent::Finished {
            assistant_message_id: active_job.assistant_message_id,
            vendor_id: active_job.vendor_id,
        });

        let surface = state.surface();
        assert!(!surface.composer.stop_enabled);
        assert_eq!(
            surface
                .message_list
                .messages
                .last()
                .map(|it| it.body.as_str()),
            Some("新しい応答")
        );
        Ok(())
    }

    fn temp_attachment_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }
}
