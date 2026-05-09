use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use katana_chat_ui::{
    Attachment, ChatOutputKind, ChatSession, ChatSessionError, ChatSettingsError, ChatTextKey,
    ChatUiConfig, ChatUiSurface, ContextUsageSnapshot, DiffCandidateOutput, FileCandidateOutput,
    FileResource, OutputStatus, TextCatalog, ThinkingLog, VendorUiState,
};

use crate::provider::{
    ManualProviderEvent, ManualProviderExecution, ManualProviderJob, ManualProviderRegistry,
    current_working_dir,
};

#[derive(Debug)]
pub(crate) enum ManualFloemError {
    Session(ChatSessionError),
    Settings(ChatSettingsError),
}

impl fmt::Display for ManualFloemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Session(error) => write!(formatter, "{error}"),
            Self::Settings(error) => write!(formatter, "{error}"),
        }
    }
}

impl From<ChatSessionError> for ManualFloemError {
    fn from(error: ChatSessionError) -> Self {
        Self::Session(error)
    }
}

impl From<ChatSettingsError> for ManualFloemError {
    fn from(error: ChatSettingsError) -> Self {
        Self::Settings(error)
    }
}

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
        let mut session = ChatSession::with_config(manual_host_config())?;
        session.set_context_usage(ContextUsageSnapshot::new(0, 200_000));
        let providers = ManualProviderRegistry::discover();
        let last_event = configure_provider_state(&mut session, &providers);
        session.set_text_catalog(TextCatalog::english().with_text(ChatTextKey::SendButton, "Run"));
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

    pub(crate) fn attach_dialog_opening(&mut self) {
        self.last_event = "添付ファイルを選択中です".to_string();
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

    pub(crate) fn start_new_chat(&mut self) {
        let providers = self.providers.clone();
        let session = match ChatSession::with_config(manual_host_config()) {
            Ok(session) => session,
            Err(error) => {
                self.last_event = format!("新しい会話の開始失敗: {error}");
                return;
            }
        };
        self.session = session;
        self.session
            .set_context_usage(ContextUsageSnapshot::new(0, 200_000));
        configure_provider_state(&mut self.session, &providers);
        self.active_assistant_message_id = None;
        self.title_updated = false;
        self.last_event = "新しい会話を開始しました".to_string();
    }

    pub(crate) fn open_history(&mut self) {
        self.last_event = "履歴を開きました".to_string();
    }

    pub(crate) fn select_vendor(&mut self, vendor_id: String) {
        let Some(state) = self.providers.vendor_state_for(&vendor_id) else {
            self.last_event = format!("利用できない provider です: {vendor_id}");
            return;
        };
        self.apply_provider_connection_state(&state);
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
            ManualProviderEvent::Output {
                assistant_message_id,
                vendor_id,
                kind,
            } => self.apply_provider_output(assistant_message_id, vendor_id, kind),
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

    pub(crate) fn undo_output(&mut self, output_id: u64) {
        match self.undo_output_result(output_id) {
            Ok(()) => self.last_event = format!("output を取り消しました: {output_id}"),
            Err(error) => self.last_event = format!("output 取り消し失敗: {error}"),
        }
    }

    pub(crate) fn handle_output_action(
        &mut self,
        output_id: u64,
        action: katana_chat_ui::HostActionKind,
    ) {
        match action {
            katana_chat_ui::HostActionKind::UndoChange => self.undo_output(output_id),
            _ => self.last_event = format!("output action: {action:?} ({output_id})"),
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
        self.ensure_submit_can_start()?;
        let vendor_state = self.session.vendor_ui_state().clone();
        self.ensure_provider_available(&vendor_state.active_vendor_id)?;
        self.providers.validate_model(
            &vendor_state.active_vendor_id,
            vendor_state.selected_model.as_deref(),
        )?;
        let execution = self.provider_execution(&vendor_state.active_vendor_id)?;
        self.session.draft_mut().set_text(text);
        let prompt = Self::provider_prompt(self.session.draft());
        self.session
            .submit_draft()
            .map_err(|error| error.to_string())?;
        let assistant_id = self.start_assistant_response(&vendor_state)?;
        self.active_assistant_message_id = Some(assistant_id);
        Self::provider_job(assistant_id, vendor_state, execution, prompt)
    }

    fn ensure_submit_can_start(&self) -> Result<(), String> {
        if self.active_assistant_message_id.is_some() {
            return Err("provider response is already running".to_string());
        }
        Ok(())
    }

    fn ensure_provider_available(&self, vendor_id: &str) -> Result<(), String> {
        if vendor_id.is_empty() {
            return Err("provider is not configured".to_string());
        }
        if let Some(reason) = self.providers.unavailable_reason_for(vendor_id) {
            return Err(reason);
        }
        if self.providers.contains(vendor_id) {
            return Ok(());
        }
        Err(format!("provider is not available: {vendor_id}"))
    }

    fn provider_execution(&self, vendor_id: &str) -> Result<ManualProviderExecution, String> {
        self.providers
            .execution_for(vendor_id)
            .ok_or_else(|| format!("provider execution is not configured: {vendor_id}"))
    }

    fn start_assistant_response(&mut self, vendor_state: &VendorUiState) -> Result<u64, String> {
        match Self::thinking_log_start(vendor_state) {
            Some(thinking) => self
                .session
                .start_assistant_stream_with_thinking(String::new(), thinking),
            None => self.session.start_assistant_stream(String::new()),
        }
        .map_err(|error| error.to_string())
    }

    fn provider_job(
        assistant_id: u64,
        vendor_state: VendorUiState,
        execution: ManualProviderExecution,
        prompt: String,
    ) -> Result<ManualProviderJob, String> {
        Ok(ManualProviderJob {
            assistant_message_id: assistant_id,
            vendor_id: vendor_state.active_vendor_id,
            execution,
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

    fn apply_provider_output(
        &mut self,
        assistant_id: u64,
        vendor_id: String,
        kind: katana_chat_ui::ChatOutputKind,
    ) {
        if self.reject_inactive(assistant_id, &vendor_id) {
            return;
        }
        let candidate = ManualOutputCandidate::from_kind(&kind);
        match self.session.add_output(assistant_id, kind) {
            Ok(output_id) => {
                self.apply_output_candidate(output_id, candidate);
                self.last_event = format!("{vendor_id} output を受信しました");
            }
            Err(error) => self.last_event = format!("{vendor_id} output 反映失敗: {error}"),
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

    fn apply_provider_connection_state(&mut self, state: &VendorUiState) {
        match self
            .providers
            .unavailable_reason_for(&state.active_vendor_id)
        {
            Some(reason) => self.session.set_provider_missing(reason),
            None => self
                .session
                .set_provider_configured(state.active_vendor_id.clone()),
        }
    }

    fn apply_output_candidate(&mut self, output_id: u64, candidate: Option<ManualOutputCandidate>) {
        let Some(candidate) = candidate else {
            return;
        };
        let status = match Self::apply_candidate(candidate) {
            Ok(_) => OutputStatus::Applied,
            Err(reason) => OutputStatus::Failed(reason),
        };
        if let Err(error) = self.session.set_output_status(output_id, status) {
            self.last_event = format!("output status 反映失敗: {error}");
        }
    }

    fn apply_candidate(candidate: ManualOutputCandidate) -> Result<PathBuf, String> {
        let cwd = current_working_dir()?;
        let cwd = Path::new(&cwd);
        match candidate {
            ManualOutputCandidate::File(file) => ManualFileCreateAction::apply(cwd, &file),
            ManualOutputCandidate::Diff(diff) => ManualDiffApplyAction::apply(cwd, &diff),
        }
    }

    fn undo_output_result(&mut self, output_id: u64) -> Result<(), String> {
        let output = self
            .session
            .output(output_id)
            .map_err(|error| error.to_string())?
            .clone();
        let candidate = ManualOutputCandidate::from_kind(&output.kind)
            .ok_or_else(|| format!("output is not undoable: {output_id}"))?;
        Self::undo_candidate(candidate)?;
        self.session
            .set_output_status(output_id, OutputStatus::Reverted)
            .map_err(|error| error.to_string())
    }

    fn undo_candidate(candidate: ManualOutputCandidate) -> Result<PathBuf, String> {
        let cwd = current_working_dir()?;
        let cwd = Path::new(&cwd);
        match candidate {
            ManualOutputCandidate::File(file) => ManualFileCreateAction::undo(cwd, &file),
            ManualOutputCandidate::Diff(diff) => ManualDiffApplyAction::undo(cwd, &diff),
        }
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
            .map_or_else(String::new, ToString::to_string)
            .chars()
            .take(42)
            .collect()
    }

    #[cfg(test)]
    fn new_for_test() -> Result<Self, ManualFloemError> {
        Self::new_for_test_with_providers(ManualProviderRegistry::for_test_katanagent_agent())
    }

    #[cfg(test)]
    fn new_for_test_with_providers(
        providers: ManualProviderRegistry,
    ) -> Result<Self, ManualFloemError> {
        let mut session = ChatSession::new();
        session.apply_config(manual_host_config())?;
        session.set_context_usage(ContextUsageSnapshot::new(0, 200_000));
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
        self.providers = providers;
        self.last_event = configure_provider_state(&mut self.session, &self.providers);
    }
}

fn manual_host_config() -> ChatUiConfig {
    ChatUiConfig::default()
}

fn configure_provider_state(
    session: &mut ChatSession,
    providers: &ManualProviderRegistry,
) -> String {
    match providers.first_vendor_state() {
        Some(state) => {
            match providers.unavailable_reason_for(&state.active_vendor_id) {
                Some(reason) => session.set_provider_missing(reason),
                None => session.set_provider_configured(state.active_vendor_id.clone()),
            }
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

enum ManualOutputCandidate {
    File(FileCandidateOutput),
    Diff(DiffCandidateOutput),
}

impl ManualOutputCandidate {
    fn from_kind(kind: &ChatOutputKind) -> Option<Self> {
        match kind {
            ChatOutputKind::FileCandidate(file) => Some(Self::File(file.clone())),
            ChatOutputKind::DiffCandidate(diff) => Some(Self::Diff(diff.clone())),
            ChatOutputKind::Text(_)
            | ChatOutputKind::Code(_)
            | ChatOutputKind::ToolResult(_)
            | ChatOutputKind::PermissionRequest(_) => None,
        }
    }
}

struct ManualFileCreateAction;

impl ManualFileCreateAction {
    fn apply(cwd: &Path, file: &FileCandidateOutput) -> Result<PathBuf, String> {
        let target_path = Self::target_path(cwd, Path::new(&file.path))?;
        let Some(parent) = target_path.parent() else {
            return Err(format!("parent directory is unavailable: {}", file.path));
        };
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        fs::write(&target_path, &file.content).map_err(|error| error.to_string())?;
        Ok(target_path)
    }

    fn undo(cwd: &Path, file: &FileCandidateOutput) -> Result<PathBuf, String> {
        let target_path = Self::target_path(cwd, Path::new(&file.path))?;
        if !target_path.is_file() {
            return Err(format!(
                "created file is unavailable: {}",
                target_path.display()
            ));
        }
        fs::remove_file(&target_path).map_err(|error| error.to_string())?;
        Ok(target_path)
    }

    fn target_path(cwd: &Path, requested_path: &Path) -> Result<PathBuf, String> {
        let cwd = Self::normalize(cwd)?;
        let tmp_root = Self::normalize(&cwd.join("tmp"))?;
        let joined = if requested_path.is_absolute() {
            requested_path.to_path_buf()
        } else {
            cwd.join(requested_path)
        };
        let target_path = Self::normalize(&joined)?;
        if target_path.starts_with(&tmp_root) {
            return Ok(target_path);
        }
        Err(format!(
            "manual host can create files only under {}: {}",
            tmp_root.display(),
            requested_path.display()
        ))
    }

    fn normalize(path: &Path) -> Result<PathBuf, String> {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
                std::path::Component::RootDir => normalized.push(Path::new("/")),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if !normalized.pop() {
                        return Err(format!("path escapes root: {}", path.display()));
                    }
                }
                std::path::Component::Normal(part) => normalized.push(part),
            }
        }
        Ok(normalized)
    }
}

struct ManualDiffApplyAction;

impl ManualDiffApplyAction {
    fn apply(cwd: &Path, diff: &DiffCandidateOutput) -> Result<PathBuf, String> {
        Self::write_content(cwd, diff, &diff.updated_content)
    }

    fn undo(cwd: &Path, diff: &DiffCandidateOutput) -> Result<PathBuf, String> {
        Self::write_content(cwd, diff, &diff.original_content)
    }

    fn write_content(
        cwd: &Path,
        diff: &DiffCandidateOutput,
        content: &str,
    ) -> Result<PathBuf, String> {
        let target_path = ManualFileCreateAction::target_path(cwd, Path::new(&diff.target_path))?;
        if !target_path.is_file() {
            return Err(format!(
                "target file is unavailable: {}",
                target_path.display()
            ));
        }
        fs::write(&target_path, content).map_err(|error| error.to_string())?;
        Ok(target_path)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ATTACHMENT_TRUNCATED_NOTICE, MAX_ATTACHMENT_PROMPT_CHARS, ManualDiffApplyAction,
        ManualFileCreateAction, ManualFloemState, ManualProviderRegistry,
    };
    use crate::provider::{ManualProviderEvent, ManualProviderJob};
    use katana_chat_ui::{
        Attachment, ChatOutputKind, ChatSessionError, ChatUiSurface, DiffCandidateOutput,
        FileCandidateOutput, OutputStatus,
    };
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
    fn submit_adds_user_turn_and_streaming_stop_state() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let before_count = state.surface().message_list.messages.len();

        let job = state.start_submit("送信確認".to_string());

        let surface = state.surface();
        assert_eq!(surface.message_list.messages.len(), before_count + 2);
        assert!(surface.composer.stop_enabled);
        assert!(job.is_some());
        assert_eq!(state.last_event, "送信しました: katanagent 応答待ち");
        Ok(())
    }

    #[test]
    fn katanagent_agent_provider_keeps_ollama_as_runtime_model_label()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test_with_providers(
            ManualProviderRegistry::for_test_katanagent_agent(),
        )?;

        let job = start_required_job(&mut state, "README の下書きを作って")?;

        let surface = state.surface();
        assert_eq!(surface.vendor_bar.active_vendor_id, "katanagent");
        assert_eq!(surface.vendor_bar.active_vendor_label, "KatanAgent");
        assert_eq!(job.vendor_id, "katanagent");
        assert_eq!(job.model.as_deref(), Some("ollama:gemma4:e4b"));
        assert_eq!(job.permission.as_deref(), Some("default"));
        assert!(
            surface
                .vendor_bar
                .controls
                .iter()
                .any(|control| control.key == "permission")
        );
        assert_eq!(state.last_event, "送信しました: katanagent 応答待ち");
        Ok(())
    }

    #[test]
    fn select_vendor_rejects_unavailable_provider() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;

        state.select_vendor("missing-provider".to_string());

        assert_eq!(
            state.session.vendor_ui_state().active_vendor_id,
            "katanagent".to_string()
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
    fn unavailable_katanagent_ollama_catalog_disables_submit() -> Result<(), super::ManualFloemError>
    {
        let mut state = ManualFloemState::new_for_test_with_providers(
            ManualProviderRegistry::for_test_katanagent_unavailable(),
        )?;

        let job = state.start_submit("送信できないはず".to_string());

        let surface = state.surface();
        assert!(job.is_none());
        assert!(surface.vendor_bar.active_vendor_id.is_empty());
        assert!(!surface.composer.send_available);
        assert_eq!(state.last_event, "送信失敗: provider is not configured");
        Ok(())
    }

    #[test]
    fn provider_events_finish_streaming_state() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let job = start_required_job(&mut state, "送信確認")?;
        let before_messages = state.surface().message_list.messages;

        apply_chunk(&mut state, &job, "応答しました");
        state.apply_provider_event(ManualProviderEvent::Finished {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
        });

        let surface = state.surface();
        assert_eq!(surface.message_list.messages.len(), before_messages.len());
        assert_eq!(last_message_outputs_empty(&surface), Some(true));
        assert!(!surface.composer.stop_enabled);
        assert_eq!(last_message_body(&surface), Some("応答しました"));
        assert_eq!(last_message_thinking_completed(&surface), None);
        Ok(())
    }

    #[test]
    fn provider_output_is_exposed_as_handoff_not_thread_body() -> Result<(), super::ManualFloemError>
    {
        let mut state = ManualFloemState::new_for_test()?;
        let job = start_required_job(&mut state, "編集して")?;

        state.apply_provider_event(ManualProviderEvent::Output {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
            kind: ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                "/tmp/generated.md",
                "before",
                "after",
                "--- a/generated.md\n+++ b/generated.md\n@@ -1 +1 @@\n-before\n+after",
                "/tmp/generated.md を更新",
            )),
        });

        let surface = state.surface();
        assert_eq!(surface.output_handoff.outputs.len(), 1);
        assert_eq!(last_message_outputs_empty(&surface), Some(true));
        assert_eq!(state.last_event, "katanagent output を受信しました");
        Ok(())
    }

    #[test]
    fn provider_file_output_can_be_undone_after_auto_apply() -> Result<(), String> {
        let mut state = ManualFloemState::new_for_test().map_err(|it| it.to_string())?;
        let job = start_required_job(&mut state, "作成して").map_err(|it| it.to_string())?;
        let target_path = current_tmp_test_path("kcu-manual-state-file-undo.md")?;
        remove_file_if_exists(&target_path)?;

        state.apply_provider_event(ManualProviderEvent::Output {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            kind: ChatOutputKind::FileCandidate(FileCandidateOutput::new(
                "tmp/kcu-manual-state-file-undo.md",
                "text/markdown",
                "# generated",
            )),
        });
        let output_id = state.surface().output_handoff.outputs[0].id;

        assert_file_content(&target_path, "# generated")?;
        state.undo_output(output_id);

        assert!(!target_path.exists());
        assert_output_reverted(&state, output_id)?;
        Ok(())
    }

    #[test]
    fn provider_diff_output_can_be_undone_after_auto_apply() -> Result<(), String> {
        let mut state = ManualFloemState::new_for_test().map_err(|it| it.to_string())?;
        let job = start_required_job(&mut state, "編集して").map_err(|it| it.to_string())?;
        let target_path = current_tmp_test_path("kcu-manual-state-diff-undo.md")?;
        fs::write(&target_path, "before").map_err(|it| it.to_string())?;

        state.apply_provider_event(ManualProviderEvent::Output {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            kind: ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                "tmp/kcu-manual-state-diff-undo.md",
                "before",
                "after",
                "--- a/tmp/kcu-manual-state-diff-undo.md\n+++ b/tmp/kcu-manual-state-diff-undo.md",
                "tmp/kcu-manual-state-diff-undo.md を更新",
            )),
        });
        let output_id = state.surface().output_handoff.outputs[0].id;

        assert_file_content(&target_path, "after")?;
        state.undo_output(output_id);

        assert_file_content(&target_path, "before")?;
        assert_output_reverted(&state, output_id)?;
        remove_file_if_exists(&target_path)?;
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
        let job = start_required_job(&mut state, "送信確認")?;

        state.apply_provider_event(ManualProviderEvent::Failed {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
            error: "connection failed".to_string(),
        });

        let surface = state.surface();
        assert!(!surface.composer.stop_enabled);
        assert_eq!(
            surface.message_list.messages.last().map(|it| &it.status),
            Some(&katana_chat_ui::MessageStatus::Error(
                "応答失敗: connection failed".to_string()
            ))
        );
        assert_eq!(state.last_event, "katanagent 応答失敗: connection failed");
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

        let job = start_required_job(&mut state, "添付確認")?;

        assert!(job.prompt.contains(ATTACHMENT_TRUNCATED_NOTICE));
        Ok(())
    }

    #[test]
    fn refresh_provider_registry_keeps_standard_send_icon() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let before = state.surface().composer.send.icon.svg;

        state.refresh_with_providers_for_test(ManualProviderRegistry::for_test_agent());

        assert_eq!(state.surface().composer.send.icon.svg, before);
        assert!(!state.surface().composer.send.icon.svg.contains("send-alt"));
        Ok(())
    }

    #[test]
    fn stopped_provider_result_is_not_applied_to_next_message()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let stopped_job = start_required_job(&mut state, "古い送信")?;
        state.stop();
        let active_job = start_required_job(&mut state, "新しい送信")?;

        apply_chunk(&mut state, &stopped_job, "古い応答");

        let surface = state.surface();
        assert!(surface.composer.stop_enabled);
        assert_eq!(
            surface.message_list.messages.last().map(|it| it.id),
            Some(active_job.assistant_message_id)
        );
        assert_eq!(last_message_body(&surface), Some(""));
        assert_eq!(last_message_has_provider_dump(&surface), None);
        Ok(())
    }

    #[test]
    fn selected_file_can_be_attached_and_removed() -> Result<(), String> {
        let mut state = ManualFloemState::new_for_test().map_err(|it| it.to_string())?;
        let path = temp_attachment_path("kcu-manual-attach.md");
        fs::write(&path, "# selected").map_err(|it| it.to_string())?;

        state.attach_dialog_opening();
        assert_eq!(state.last_event, "添付ファイルを選択中です");

        state.attach_selected_file_for_test(path.clone());
        let attached_surface = state.surface();
        assert_eq!(attached_surface.composer.attachments.len(), 1);
        assert_eq!(state.last_event, "添付を追加しました: 1 件");

        state.remove_attachment(0);

        let surface = state.surface();
        assert!(surface.composer.attachments.is_empty());
        assert_eq!(state.last_event, "添付を削除しました");
        fs::remove_file(path).map_err(|it| it.to_string())?;
        Ok(())
    }

    #[test]
    fn manual_file_create_allows_only_tmp_under_cwd() -> Result<(), String> {
        let cwd = temp_attachment_path("kcu-manual-create-root");
        let target_file = cwd.join("tmp").join("sample.md");
        fs::create_dir_all(&cwd).map_err(|it| it.to_string())?;

        let created_path = ManualFileCreateAction::apply(
            &cwd,
            &FileCandidateOutput::new("tmp/sample.md", "text/markdown", "# sample"),
        )?;

        assert_eq!(created_path, target_file);
        assert_eq!(
            fs::read_to_string(&target_file).map_err(|it| it.to_string())?,
            "# sample"
        );
        fs::remove_file(target_file).map_err(|it| it.to_string())?;
        fs::remove_dir_all(cwd).map_err(|it| it.to_string())?;
        Ok(())
    }

    #[test]
    fn manual_file_create_can_undo_created_tmp_file() -> Result<(), String> {
        let cwd = temp_attachment_path("kcu-manual-create-undo");
        let file = FileCandidateOutput::new("tmp/sample.md", "text/markdown", "# sample");
        fs::create_dir_all(&cwd).map_err(|it| it.to_string())?;

        let target_file = ManualFileCreateAction::apply(&cwd, &file)?;
        ManualFileCreateAction::undo(&cwd, &file)?;

        assert!(!target_file.exists());
        fs::remove_dir_all(cwd).map_err(|it| it.to_string())?;
        Ok(())
    }

    #[test]
    fn manual_file_create_rejects_parent_path() -> Result<(), String> {
        let cwd = temp_attachment_path("kcu-manual-create-reject");
        fs::create_dir_all(&cwd).map_err(|it| it.to_string())?;

        let result = ManualFileCreateAction::apply(
            &cwd,
            &FileCandidateOutput::new("../sample.md", "text/markdown", "# sample"),
        );

        assert!(result.is_err());
        fs::remove_dir_all(cwd).map_err(|it| it.to_string())?;
        Ok(())
    }

    #[test]
    fn manual_diff_apply_can_restore_original_content() -> Result<(), String> {
        let cwd = temp_attachment_path("kcu-manual-diff-undo");
        let target_file = cwd.join("tmp").join("sample.md");
        fs::create_dir_all(target_file.parent().ok_or("missing parent")?)
            .map_err(|it| it.to_string())?;
        fs::write(&target_file, "before").map_err(|it| it.to_string())?;
        let diff = DiffCandidateOutput::new(
            "tmp/sample.md",
            "before",
            "after",
            "--- a/tmp/sample.md\n+++ b/tmp/sample.md\n@@ -1 +1 @@\n-before\n+after",
            "tmp/sample.md を更新",
        );

        ManualDiffApplyAction::apply(&cwd, &diff)?;
        assert_eq!(
            fs::read_to_string(&target_file).map_err(|it| it.to_string())?,
            "after"
        );

        ManualDiffApplyAction::undo(&cwd, &diff)?;
        assert_eq!(
            fs::read_to_string(&target_file).map_err(|it| it.to_string())?,
            "before"
        );
        fs::remove_dir_all(cwd).map_err(|it| it.to_string())?;
        Ok(())
    }

    #[test]
    fn new_chat_clears_session_without_losing_provider() -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        state.start_submit("古い会話".to_string());

        state.start_new_chat();

        let surface = state.surface();
        assert!(surface.message_list.messages.is_empty());
        assert_eq!(surface.vendor_bar.active_vendor_id, "katanagent");
        assert_eq!(state.last_event, "新しい会話を開始しました");
        Ok(())
    }

    #[test]
    fn active_provider_result_after_stopped_result_is_applied()
    -> Result<(), super::ManualFloemError> {
        let mut state = ManualFloemState::new_for_test()?;
        let stopped_job = start_required_job(&mut state, "古い送信")?;
        state.stop();
        let active_job = start_required_job(&mut state, "新しい送信")?;

        apply_chunk(&mut state, &stopped_job, "古い応答");
        apply_chunk(&mut state, &active_job, "新しい応答");
        state.apply_provider_event(ManualProviderEvent::Finished {
            assistant_message_id: active_job.assistant_message_id,
            vendor_id: active_job.vendor_id,
        });

        let surface = state.surface();
        assert!(!surface.composer.stop_enabled);
        assert_eq!(last_message_body(&surface), Some("新しい応答"));
        Ok(())
    }

    fn start_required_job(
        state: &mut ManualFloemState,
        prompt: &str,
    ) -> Result<ManualProviderJob, super::ManualFloemError> {
        state
            .start_submit(prompt.to_string())
            .ok_or(super::ManualFloemError::Session(
                ChatSessionError::DraftEmpty,
            ))
    }

    fn apply_chunk(state: &mut ManualFloemState, job: &ManualProviderJob, content: &str) {
        state.apply_provider_event(ManualProviderEvent::Chunk {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            content: content.to_string(),
        });
    }

    fn last_message_body(surface: &ChatUiSurface) -> Option<&str> {
        surface
            .message_list
            .messages
            .last()
            .map(|message| message.body.as_str())
    }

    fn last_message_outputs_empty(surface: &ChatUiSurface) -> Option<bool> {
        surface
            .message_list
            .messages
            .last()
            .map(|message| message.outputs.is_empty())
    }

    fn last_message_thinking_completed(surface: &ChatUiSurface) -> Option<bool> {
        surface
            .message_list
            .messages
            .last()
            .and_then(|message| message.thinking.as_ref())
            .map(|thinking| thinking.completed)
    }

    fn last_message_has_provider_dump(surface: &ChatUiSurface) -> Option<bool> {
        surface
            .message_list
            .messages
            .last()
            .and_then(|message| message.thinking.as_ref())
            .map(|thinking| {
                thinking
                    .entries
                    .iter()
                    .any(|entry| entry == "provider: claude-code")
            })
    }

    fn temp_attachment_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    fn current_tmp_test_path(name: &str) -> Result<PathBuf, String> {
        let path = std::env::current_dir()
            .map_err(|it| it.to_string())?
            .join("tmp")
            .join(name);
        let Some(parent) = path.parent() else {
            return Err("missing tmp parent".to_string());
        };
        fs::create_dir_all(parent).map_err(|it| it.to_string())?;
        Ok(path)
    }

    fn remove_file_if_exists(path: &PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Ok(());
        }
        fs::remove_file(path).map_err(|it| it.to_string())
    }

    fn assert_file_content(path: &PathBuf, expected: &str) -> Result<(), String> {
        assert_eq!(
            fs::read_to_string(path).map_err(|it| it.to_string())?,
            expected
        );
        Ok(())
    }

    fn assert_output_reverted(state: &ManualFloemState, output_id: u64) -> Result<(), String> {
        assert_eq!(
            state
                .session
                .output(output_id)
                .map(|it| it.status.clone())
                .map_err(|it| it.to_string())?,
            OutputStatus::Reverted
        );
        Ok(())
    }
}
