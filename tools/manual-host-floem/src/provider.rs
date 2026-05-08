use std::{
    env,
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use crossbeam_channel::Sender;
use katana_acp_client::{
    AcpError, AiIntent, AiProvider, AiRequest, AiStreamEvent, ChatRole, ChatTurn, DocumentContext,
    ollama::OllamaProvider,
};
use katana_chat_ui::{ChatOutputKind, FileCandidateOutput, VendorUiState};
#[cfg(test)]
use katana_chat_ui::{DiffCandidateOutput, PermissionRequestOutput, ToolResultOutput};
use serde::Deserialize;

const KATANAGENT_ID: &str = "katanagent";
const CLAUDE_CODE_ID: &str = "claude-code";
const CODEX_CLI_ID: &str = "codex-cli";
const GITHUB_COPILOT_ID: &str = "github-copilot";
const OPENCODE_ID: &str = "opencode";
const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";
const KCU_OLLAMA_ENDPOINT_ENV: &str = "KCU_OLLAMA_ENDPOINT";
const KCU_OLLAMA_MODEL_ENV: &str = "KCU_OLLAMA_MODEL";
const OLLAMA_MODEL_PREFIX: &str = "ollama:";
const SAMPLE_MARKDOWN_PATH: &str = "tmp/sample.md";

#[derive(Clone)]
pub(crate) struct ManualProviderRegistry {
    providers: Vec<ManualProviderDescriptor>,
}

impl ManualProviderRegistry {
    pub(crate) fn discover() -> Self {
        let mut providers = Vec::new();
        providers.push(ManualProviderDescriptor::for_katanagent_from_environment());
        Self::push_command_provider(&mut providers, CLAUDE_CODE_ID, "claude");
        Self::push_command_provider(&mut providers, CODEX_CLI_ID, "codex");
        Self::push_github_copilot(&mut providers);
        Self::push_command_provider(&mut providers, OPENCODE_ID, "opencode");
        Self { providers }
    }

    #[cfg(test)]
    pub(crate) fn for_test_agent() -> Self {
        Self {
            providers: vec![ManualProviderDescriptor::from_detected_command(
                CLAUDE_CODE_ID,
            )],
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test_katanagent_agent() -> Self {
        Self {
            providers: vec![ManualProviderDescriptor::for_katanagent_with_models(
                DEFAULT_OLLAMA_ENDPOINT.to_string(),
                vec!["gemma4:e4b".to_string(), "llama3".to_string()],
                None,
            )],
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test_katanagent_unavailable() -> Self {
        Self {
            providers: vec![ManualProviderDescriptor::for_unavailable_katanagent(
                DEFAULT_OLLAMA_ENDPOINT.to_string(),
                "Ollama model catalog is unavailable",
            )],
        }
    }

    #[cfg(test)]
    pub(crate) fn empty_for_test() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub(crate) fn available_vendor_ids(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|provider| provider.unavailable_reason.is_none())
            .map(|provider| provider.vendor_id.clone())
            .collect()
    }

    pub(crate) fn first_vendor_state(&self) -> Option<VendorUiState> {
        self.providers
            .iter()
            .find(|provider| provider.unavailable_reason.is_none())
            .map(|provider| self.vendor_state(provider))
    }

    pub(crate) fn vendor_state_for(&self, vendor_id: &str) -> Option<VendorUiState> {
        self.providers
            .iter()
            .find(|provider| {
                provider.vendor_id == vendor_id && provider.unavailable_reason.is_none()
            })
            .map(|provider| self.vendor_state(provider))
    }

    pub(crate) fn contains(&self, vendor_id: &str) -> bool {
        self.providers
            .iter()
            .any(|provider| provider.vendor_id == vendor_id)
    }

    pub(crate) fn unavailable_reason_for(&self, vendor_id: &str) -> Option<String> {
        self.providers
            .iter()
            .find(|provider| provider.vendor_id == vendor_id)
            .and_then(|provider| provider.unavailable_reason.clone())
    }

    pub(crate) fn execution_for(&self, vendor_id: &str) -> Option<ManualProviderExecution> {
        self.providers
            .iter()
            .find(|provider| provider.vendor_id == vendor_id)
            .map(|provider| provider.execution)
    }

    pub(crate) fn validate_model(
        &self,
        vendor_id: &str,
        model: Option<&str>,
    ) -> Result<(), String> {
        let Some(provider) = self
            .providers
            .iter()
            .find(|provider| provider.vendor_id == vendor_id)
        else {
            return Err(format!("provider is not available: {vendor_id}"));
        };
        if let Some(reason) = &provider.unavailable_reason {
            return Err(reason.clone());
        }
        provider.validate_model(model)
    }

    fn vendor_state(&self, provider: &ManualProviderDescriptor) -> VendorUiState {
        let mut state = VendorUiState::for_vendor(provider.vendor_id.clone())
            .with_available_vendors(self.available_vendor_ids());
        if let Some(endpoint) = &provider.endpoint {
            state = state.with_endpoint(endpoint.clone());
        }
        if let Some(model) = provider.selected_model() {
            state = state.with_models(provider.model_options.clone(), model);
        }
        if let Some(thinking) = provider.selected_thinking() {
            state = state.with_thinking(provider.thinking_options.clone(), thinking);
        }
        if let Some(permission) = provider.selected_permission() {
            state = state.with_permission_modes(provider.permission_options.clone(), permission);
        }
        state
    }

    fn push_command_provider(
        providers: &mut Vec<ManualProviderDescriptor>,
        vendor_id: &str,
        command: &str,
    ) {
        if !command_available(command) {
            return;
        }
        providers.push(ManualProviderDescriptor::from_detected_command(vendor_id));
    }

    fn push_github_copilot(providers: &mut Vec<ManualProviderDescriptor>) {
        if !github_copilot_available() {
            return;
        }
        providers.push(ManualProviderDescriptor::from_detected_command(
            GITHUB_COPILOT_ID,
        ));
    }
}

#[derive(Clone)]
struct ManualProviderDescriptor {
    vendor_id: String,
    endpoint: Option<String>,
    model_options: Vec<String>,
    thinking_options: Vec<String>,
    permission_options: Vec<String>,
    execution: ManualProviderExecution,
    unavailable_reason: Option<String>,
}

impl ManualProviderDescriptor {
    fn from_detected_command(vendor_id: &str) -> Self {
        match vendor_id {
            KATANAGENT_ID => Self::for_katanagent_from_environment(),
            CLAUDE_CODE_ID => Self::for_claude_code(),
            CODEX_CLI_ID => Self::for_codex_cli(),
            OPENCODE_ID => Self::for_opencode(),
            _ => Self::unknown(vendor_id),
        }
    }

    fn for_katanagent_from_environment() -> Self {
        let endpoint = ollama_endpoint();
        match OllamaRuntimeCatalog::from_environment(endpoint.clone()) {
            Ok(catalog) => Self::for_katanagent_with_catalog(catalog),
            Err(error) => Self::for_unavailable_katanagent(endpoint, error),
        }
    }

    fn for_katanagent_with_catalog(catalog: OllamaRuntimeCatalog) -> Self {
        Self {
            vendor_id: KATANAGENT_ID.to_string(),
            endpoint: Some(catalog.endpoint),
            model_options: catalog.model_options,
            thinking_options: vec![
                "false".to_string(),
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
            ],
            permission_options: vec!["default".to_string(), "ask".to_string(), "auto".to_string()],
            execution: ManualProviderExecution::KatanAgentOllamaAgent,
            unavailable_reason: None,
        }
    }

    fn for_unavailable_katanagent(
        endpoint: String,
        reason: impl Into<String>,
    ) -> ManualProviderDescriptor {
        Self {
            vendor_id: KATANAGENT_ID.to_string(),
            endpoint: Some(endpoint),
            model_options: Vec::new(),
            thinking_options: vec!["false".to_string()],
            permission_options: vec!["default".to_string()],
            execution: ManualProviderExecution::Unavailable,
            unavailable_reason: Some(reason.into()),
        }
    }

    #[cfg(test)]
    fn for_katanagent_with_models(
        endpoint: String,
        models: Vec<String>,
        requested_model: Option<String>,
    ) -> Self {
        Self::for_katanagent_with_catalog(OllamaRuntimeCatalog::from_models(
            endpoint,
            models,
            requested_model,
        ))
    }

    fn for_claude_code() -> Self {
        Self {
            vendor_id: CLAUDE_CODE_ID.to_string(),
            endpoint: None,
            model_options: vec![
                "claude-sonnet-4-6".to_string(),
                "claude-haiku-4-5".to_string(),
            ],
            thinking_options: vec![
                "default".to_string(),
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
                "xhigh".to_string(),
                "max".to_string(),
            ],
            permission_options: vec![
                "default".to_string(),
                "auto".to_string(),
                "plan".to_string(),
            ],
            execution: ManualProviderExecution::Unavailable,
            unavailable_reason: Some("v0.1.0 harness uses KatanAgent + Ollama runtime".to_string()),
        }
    }

    fn for_codex_cli() -> Self {
        let config = CodexCliConfig::load();
        let model_options = config.model.clone().into_iter().collect::<Vec<_>>();
        let thinking_options = config
            .model_reasoning_effort
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        let permission_options = config
            .approval_policy
            .clone()
            .into_iter()
            .chain([
                "untrusted".to_string(),
                "on-request".to_string(),
                "never".to_string(),
            ])
            .collect::<Vec<_>>();
        Self {
            vendor_id: CODEX_CLI_ID.to_string(),
            endpoint: None,
            model_options: unique_values(model_options),
            thinking_options: unique_values(thinking_options),
            permission_options: unique_values(permission_options),
            execution: ManualProviderExecution::Unavailable,
            unavailable_reason: Some("v0.1.0 harness uses KatanAgent + Ollama runtime".to_string()),
        }
    }

    fn for_opencode() -> Self {
        Self {
            vendor_id: OPENCODE_ID.to_string(),
            endpoint: None,
            model_options: opencode_models(),
            thinking_options: vec!["false".to_string(), "true".to_string()],
            permission_options: Vec::new(),
            execution: ManualProviderExecution::Unavailable,
            unavailable_reason: Some("v0.1.0 harness uses KatanAgent + Ollama runtime".to_string()),
        }
    }

    fn unknown(vendor_id: &str) -> Self {
        Self {
            vendor_id: vendor_id.to_string(),
            endpoint: None,
            model_options: Vec::new(),
            thinking_options: Vec::new(),
            permission_options: Vec::new(),
            execution: ManualProviderExecution::Unavailable,
            unavailable_reason: Some("unknown provider".to_string()),
        }
    }

    fn selected_model(&self) -> Option<String> {
        self.model_options.first().cloned()
    }

    fn selected_thinking(&self) -> Option<String> {
        self.thinking_options.first().cloned()
    }

    fn selected_permission(&self) -> Option<String> {
        self.permission_options.first().cloned()
    }

    fn validate_model(&self, model: Option<&str>) -> Result<(), String> {
        let Some(model) = model else {
            return Err(format!("model is not configured for {}", self.vendor_id));
        };
        if self.model_options.iter().any(|it| it == model) {
            return Ok(());
        }
        Err(format!(
            "model is not available for {}: {}",
            self.vendor_id, model
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManualProviderExecution {
    KatanAgentOllamaAgent,
    Unavailable,
    #[cfg(test)]
    MockAgent,
}

struct OllamaRuntimeCatalog {
    endpoint: String,
    model_options: Vec<String>,
}

impl OllamaRuntimeCatalog {
    fn from_environment(endpoint: String) -> Result<Self, String> {
        let models = list_ollama_models(&endpoint)?;
        Self::try_from_models(endpoint, models, env::var(KCU_OLLAMA_MODEL_ENV).ok())
    }

    #[cfg(test)]
    fn from_models(endpoint: String, models: Vec<String>, requested_model: Option<String>) -> Self {
        let mut normalized = unique_values(models);
        if let Some(requested_model) = requested_model.map(|model| raw_ollama_model(&model))
            && let Some(index) = normalized.iter().position(|it| it == &requested_model)
        {
            let selected = normalized.remove(index);
            normalized.insert(0, selected);
        }
        let model_options = normalized.into_iter().map(ollama_model_option).collect();
        Self {
            endpoint,
            model_options,
        }
    }

    fn try_from_models(
        endpoint: String,
        models: Vec<String>,
        requested_model: Option<String>,
    ) -> Result<Self, String> {
        let model_options = Self::try_select_model_options(models, requested_model)?;
        Ok(Self {
            endpoint,
            model_options,
        })
    }

    fn try_select_model_options(
        models: Vec<String>,
        requested_model: Option<String>,
    ) -> Result<Vec<String>, String> {
        let mut normalized = unique_values(models);
        if normalized.is_empty() {
            return Err("Ollama model catalog is empty".to_string());
        }
        if let Some(requested_model) = requested_model {
            let requested_model = raw_ollama_model(&requested_model);
            if let Some(index) = normalized.iter().position(|it| it == &requested_model) {
                let selected = normalized.remove(index);
                normalized.insert(0, selected);
                return Ok(normalized.into_iter().map(ollama_model_option).collect());
            }
            return Err(format!(
                "{KCU_OLLAMA_MODEL_ENV} is not installed in Ollama: {requested_model}"
            ));
        }
        Ok(normalized.into_iter().map(ollama_model_option).collect())
    }
}

#[derive(Clone, Default, Deserialize)]
struct CodexCliConfig {
    model: Option<String>,
    model_reasoning_effort: Option<String>,
    approval_policy: Option<String>,
}

impl CodexCliConfig {
    fn load() -> Self {
        let Some(path) = codex_config_path() else {
            return Self::default();
        };
        let Ok(content) = fs::read_to_string(path) else {
            return Self::default();
        };
        if let Ok(config) = toml::from_str(&content) {
            return config;
        }
        Self::default()
    }
}

#[derive(Clone)]
pub(crate) struct ManualProviderJob {
    pub(crate) assistant_message_id: u64,
    pub(crate) vendor_id: String,
    pub(crate) execution: ManualProviderExecution,
    pub(crate) endpoint: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) thinking: Option<String>,
    pub(crate) permission: Option<String>,
    pub(crate) prompt: String,
    pub(crate) cwd: String,
}

#[derive(Clone)]
pub(crate) enum ManualProviderEvent {
    Chunk {
        assistant_message_id: u64,
        vendor_id: String,
        content: String,
    },
    ThinkingChunk {
        assistant_message_id: u64,
        vendor_id: String,
        content: String,
    },
    Output {
        assistant_message_id: u64,
        vendor_id: String,
        kind: ChatOutputKind,
    },
    Finished {
        assistant_message_id: u64,
        vendor_id: String,
    },
    Failed {
        assistant_message_id: u64,
        vendor_id: String,
        error: String,
    },
}

pub(crate) struct ManualProviderExecutor;

impl ManualProviderExecutor {
    pub(crate) fn execute(job: ManualProviderJob, sender: Sender<ManualProviderEvent>) {
        match job.execution {
            ManualProviderExecution::KatanAgentOllamaAgent => {
                Self::execute_katanagent_ollama(job, sender);
            }
            ManualProviderExecution::Unavailable => {
                Self::send_failed(job, sender, "provider execution is unavailable");
            }
            #[cfg(test)]
            ManualProviderExecution::MockAgent => Self::execute_mock(job, sender),
        }
    }

    fn execute_katanagent_ollama(job: ManualProviderJob, sender: Sender<ManualProviderEvent>) {
        let agent = match KatanAgentOllamaAgent::from_job(&job) {
            Ok(agent) => agent,
            Err(error) => {
                Self::send_failed(job, sender, error);
                return;
            }
        };
        match agent.execute(&job, &sender) {
            Ok(content) => {
                Self::emit_file_output_if_requested(&job, &sender, &content);
                Self::emit_finished(job, &sender);
            }
            Err(error) => Self::send_failed(job, sender, error.to_string()),
        }
    }

    fn send_failed(
        job: ManualProviderJob,
        sender: Sender<ManualProviderEvent>,
        error: impl Into<String>,
    ) {
        let _ = sender.send(ManualProviderEvent::Failed {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
            error: error.into(),
        });
    }

    fn emit_file_output_if_requested(
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
        content: &str,
    ) {
        let Some(target_path) = AgentFileRequest::detect_target_path(&job.prompt) else {
            return;
        };
        let file_content = MarkdownFileContent::extract(content);
        let _ = sender.send(ManualProviderEvent::Output {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            kind: ChatOutputKind::FileCandidate(FileCandidateOutput::new(
                target_path,
                "text/markdown",
                file_content,
            )),
        });
    }

    #[cfg(test)]
    fn execute_mock(job: ManualProviderJob, sender: Sender<ManualProviderEvent>) {
        if Self::emit_failure_if_needed(&job, &sender) {
            return;
        }
        Self::emit_thinking(&job, &sender);
        Self::emit_response(&job, &sender);
        Self::emit_outputs(&job, &sender);
        Self::emit_finished(job, &sender);
    }

    #[cfg(test)]
    fn emit_failure_if_needed(
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
    ) -> bool {
        if !ManualProviderMock::should_fail(job) {
            return false;
        }
        let _ = sender.send(ManualProviderEvent::Failed {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            error: "mock failure requested".to_string(),
        });
        true
    }

    #[cfg(test)]
    fn emit_thinking(job: &ManualProviderJob, sender: &Sender<ManualProviderEvent>) {
        let Some(content) = ManualProviderMock::thinking(job) else {
            return;
        };
        let _ = sender.send(ManualProviderEvent::ThinkingChunk {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            content,
        });
    }

    #[cfg(test)]
    fn emit_response(job: &ManualProviderJob, sender: &Sender<ManualProviderEvent>) {
        let _ = sender.send(ManualProviderEvent::Chunk {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            content: ManualProviderMock::response(job),
        });
    }

    #[cfg(test)]
    fn emit_outputs(job: &ManualProviderJob, sender: &Sender<ManualProviderEvent>) {
        for kind in ManualProviderMock::outputs(job) {
            let _ = sender.send(ManualProviderEvent::Output {
                assistant_message_id: job.assistant_message_id,
                vendor_id: job.vendor_id.clone(),
                kind,
            });
        }
    }

    fn emit_finished(job: ManualProviderJob, sender: &Sender<ManualProviderEvent>) {
        let _ = sender.send(ManualProviderEvent::Finished {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
        });
    }
}

struct KatanAgentOllamaAgent {
    endpoint: String,
    model: String,
    thinking: Option<String>,
}

impl KatanAgentOllamaAgent {
    fn from_job(job: &ManualProviderJob) -> Result<Self, String> {
        let endpoint = job
            .endpoint
            .clone()
            .ok_or_else(|| "Ollama endpoint is not configured".to_string())?;
        let model = job
            .model
            .as_deref()
            .map(raw_ollama_model)
            .ok_or_else(|| "Ollama model is not configured".to_string())?;
        Ok(Self {
            endpoint,
            model,
            thinking: Self::ollama_thinking_option(job.thinking.as_deref()),
        })
    }

    fn execute(
        &self,
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
    ) -> Result<String, AcpError> {
        let provider = OllamaProvider::new(Some(self.endpoint.clone()), Some(self.model.clone()))?;
        let request = Self::request(job);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| AcpError::Transport(error.to_string()))?;
        let mut content_buffer = String::new();
        runtime.block_on(provider.execute_streaming_with_thinking(
            &request,
            self.thinking.clone(),
            |event| Self::handle_stream_event(event, job, sender, &mut content_buffer),
        ))?;
        if content_buffer.trim().is_empty() {
            return Err(AcpError::Protocol("Ollama response was empty".to_string()));
        }
        Ok(content_buffer)
    }

    fn request(job: &ManualProviderJob) -> AiRequest {
        AiRequest {
            intent: AiIntent::Create,
            context: DocumentContext {
                uri: format!("file://{}/{}", job.cwd, SAMPLE_MARKDOWN_PATH),
                content: String::new(),
                cursor_offset: 0,
                diagnostics: Vec::new(),
            },
            prompt: AgentFileRequest::prompt(&job.prompt),
            history: vec![ChatTurn {
                role: ChatRole::System,
                content: agent_system_prompt(job.permission.as_deref()),
            }],
        }
    }

    fn handle_stream_event(
        event: AiStreamEvent,
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
        content_buffer: &mut String,
    ) -> Result<(), AcpError> {
        match event {
            AiStreamEvent::Content(content) => {
                content_buffer.push_str(&content);
                let _ = sender.send(ManualProviderEvent::Chunk {
                    assistant_message_id: job.assistant_message_id,
                    vendor_id: job.vendor_id.clone(),
                    content,
                });
            }
            AiStreamEvent::Thinking(content) => {
                if Self::thinking_enabled(job.thinking.as_deref()) {
                    let _ = sender.send(ManualProviderEvent::ThinkingChunk {
                        assistant_message_id: job.assistant_message_id,
                        vendor_id: job.vendor_id.clone(),
                        content,
                    });
                }
            }
        }
        Ok(())
    }

    fn ollama_thinking_option(thinking: Option<&str>) -> Option<String> {
        match thinking {
            Some("low" | "medium" | "high") => thinking.map(ToString::to_string),
            _ => Some("false".to_string()),
        }
    }

    fn thinking_enabled(thinking: Option<&str>) -> bool {
        matches!(thinking, Some("low" | "medium" | "high"))
    }
}

fn ollama_endpoint() -> String {
    match env::var(KCU_OLLAMA_ENDPOINT_ENV) {
        Ok(endpoint) if !endpoint.trim().is_empty() => endpoint,
        _ => DEFAULT_OLLAMA_ENDPOINT.to_string(),
    }
}

fn list_ollama_models(endpoint: &str) -> Result<Vec<String>, String> {
    let provider =
        OllamaProvider::new(Some(endpoint.to_string()), None).map_err(|error| error.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    runtime
        .block_on(provider.list_models())
        .map(|models| models.into_iter().map(|model| model.id).collect())
        .map_err(|error| error.to_string())
}

fn raw_ollama_model(model: &str) -> String {
    match model.strip_prefix(OLLAMA_MODEL_PREFIX) {
        Some(model) => model.to_string(),
        None => model.to_string(),
    }
}

fn ollama_model_option(model: String) -> String {
    format!("{OLLAMA_MODEL_PREFIX}{model}")
}

fn agent_system_prompt(permission: Option<&str>) -> String {
    let permission = permission.unwrap_or("default");
    [
        "あなたは katana-chat-ui の手動確認用 KatanAgent agent です。".to_string(),
        "ユーザーの指示に従い、必要なファイル本文を生成します。".to_string(),
        "ファイル作成が求められた場合は、説明ではなく対象ファイルの内容だけを返します。"
            .to_string(),
        "危険なパスや tmp 外の書き込み判断は host が行うため、本文には混ぜないでください。"
            .to_string(),
        format!("permission mode: {permission}"),
    ]
    .join("\n")
}

struct AgentFileRequest;

impl AgentFileRequest {
    fn detect_target_path(prompt: &str) -> Option<&'static str> {
        if prompt.contains("sample.md") && prompt.contains("tmp") {
            return Some(SAMPLE_MARKDOWN_PATH);
        }
        None
    }

    fn prompt(prompt: &str) -> String {
        let Some(target_path) = Self::detect_target_path(prompt) else {
            return prompt.to_string();
        };
        format!(
            "{prompt}\n\n出力対象: {target_path}\nファイル本文として使える Markdown だけを返してください。説明文やコードフェンスは付けないでください。"
        )
    }
}

struct MarkdownFileContent;

impl MarkdownFileContent {
    fn extract(content: &str) -> String {
        if let Some(fenced) = Self::first_fenced_block(content) {
            return fenced;
        }
        content.trim().to_string()
    }

    fn first_fenced_block(content: &str) -> Option<String> {
        let mut in_fence = false;
        let mut lines = Vec::new();
        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                if in_fence {
                    return Some(lines.join("\n"));
                }
                in_fence = true;
                continue;
            }
            if in_fence {
                lines.push(line);
            }
        }
        None
    }
}

#[cfg(test)]
struct ManualProviderMock;

#[cfg(test)]
impl ManualProviderMock {
    fn should_fail(job: &ManualProviderJob) -> bool {
        job.prompt.contains("__KCU_MOCK_FAIL__")
    }

    fn thinking(job: &ManualProviderJob) -> Option<String> {
        let thinking = job.thinking.as_deref()?;
        if matches!(thinking, "false" | "default") {
            return None;
        }
        Some(format!(
            "入力内容を読み取り、{} の設定に合わせて回答方針を組み立てています。",
            thinking
        ))
    }

    fn response(job: &ManualProviderJob) -> String {
        format!(
            "mock provider response\n\n- provider: {}\n- endpoint: {}\n- model: {}\n- thinking: {}\n- permission: {}\n- cwd: {}\n- prompt chars: {}\n\n```text\n{}\n```",
            job.vendor_id,
            job.endpoint.as_deref().unwrap_or("unavailable"),
            job.model.as_deref().unwrap_or("unavailable"),
            job.thinking.as_deref().unwrap_or("unavailable"),
            job.permission.as_deref().unwrap_or("unavailable"),
            job.cwd,
            job.prompt.chars().count(),
            job.prompt
        )
    }

    fn outputs(job: &ManualProviderJob) -> Vec<ChatOutputKind> {
        vec![
            ChatOutputKind::FileCandidate(FileCandidateOutput::new(
                format!("{}/tmp/kcu-generated.md", job.cwd),
                "text/markdown",
                format!("# Mock output\n\nprovider: {}\n", job.vendor_id),
            )),
            ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                format!("{}/tmp/kcu-generated.md", job.cwd),
                "--- a/tmp/kcu-generated.md\n+++ b/tmp/kcu-generated.md\n@@ -1 +1 @@\n-before\n+after",
            )),
            ChatOutputKind::ToolResult(ToolResultOutput::new(
                "mock-provider",
                format!(
                    "model={} endpoint={}",
                    job.model.as_deref().unwrap_or("unavailable"),
                    job.endpoint.as_deref().unwrap_or("unavailable")
                ),
            )),
            ChatOutputKind::PermissionRequest(PermissionRequestOutput::new(
                "mock permission",
                format!(
                    "{} requires host approval before applying generated changes",
                    job.vendor_id
                ),
            )),
        ]
    }
}

#[cfg(test)]
mod mock_tests {
    use super::{ManualProviderExecution, ManualProviderJob, ManualProviderMock};

    #[test]
    fn thinking_false_does_not_emit_thinking_log() {
        let job = job_with_thinking("false");

        assert_eq!(ManualProviderMock::thinking(&job), None);
    }

    #[test]
    fn thinking_log_is_not_provider_configuration_dump() {
        let job = job_with_thinking("high");
        let log = ManualProviderMock::thinking(&job);

        assert!(log.as_deref().is_some_and(|it| it.contains("回答方針")));
        assert!(!log.as_deref().is_some_and(|it| it.contains("cwd=")));
        assert!(!log.as_deref().is_some_and(|it| it.contains("provider=")));
    }

    fn job_with_thinking(thinking: &str) -> ManualProviderJob {
        ManualProviderJob {
            assistant_message_id: 2,
            vendor_id: "claude-code".to_string(),
            execution: ManualProviderExecution::MockAgent,
            endpoint: None,
            model: Some("claude-sonnet-4-6".to_string()),
            thinking: Some(thinking.to_string()),
            permission: Some("default".to_string()),
            prompt: "こんにちは".to_string(),
            cwd: "/tmp/kcu".to_string(),
        }
    }
}

fn command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn github_copilot_available() -> bool {
    let Ok(output) = Command::new("gh")
        .arg("extension")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout).contains("gh-copilot")
}

fn opencode_models() -> Vec<String> {
    let Ok(output) = Command::new("opencode")
        .arg("models")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|it| !it.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn unique_values(values: Vec<String>) -> Vec<String> {
    values.into_iter().fold(Vec::new(), |mut unique, value| {
        if !unique.contains(&value) {
            unique.push(value);
        }
        unique
    })
}

fn codex_config_path() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".codex").join("config.toml"))
}

pub(crate) fn current_working_dir() -> Result<String, String> {
    env::current_dir()
        .map_err(|error| error.to_string())?
        .into_os_string()
        .into_string()
        .map_err(|path| invalid_utf8_path(&path))
}

fn invalid_utf8_path(path: &OsStr) -> String {
    format!("current directory is not UTF-8: {path:?}")
}

#[cfg(test)]
mod tests {
    use super::{
        AgentFileRequest, DEFAULT_OLLAMA_ENDPOINT, KatanAgentOllamaAgent, ManualProviderEvent,
        ManualProviderExecution, ManualProviderExecutor, ManualProviderJob, ManualProviderRegistry,
        MarkdownFileContent, OllamaRuntimeCatalog, SAMPLE_MARKDOWN_PATH,
    };
    use crossbeam_channel::unbounded;
    use katana_chat_ui::ChatOutputKind;

    #[test]
    fn fixed_test_registry_excludes_ollama_and_unavailable_providers() {
        let registry = ManualProviderRegistry::for_test_agent();

        assert!(registry.available_vendor_ids().is_empty());
        assert!(!registry.contains("ollama"));
    }

    #[test]
    fn katanagent_agent_can_use_ollama_runtime_without_ollama_provider() {
        let registry = ManualProviderRegistry::for_test_katanagent_agent();
        let state = registry.first_vendor_state();

        assert!(state.is_some());
        if let Some(state) = state {
            assert_eq!(state.active_vendor_id, "katanagent");
            assert_eq!(state.selected_model.as_deref(), Some("ollama:gemma4:e4b"));
            assert_eq!(state.selected_thinking.as_deref(), Some("false"));
            assert_eq!(state.selected_permission.as_deref(), Some("default"));
        }
        assert_eq!(
            registry.execution_for("katanagent"),
            Some(ManualProviderExecution::KatanAgentOllamaAgent)
        );
        assert!(!registry.contains("ollama"));
    }

    #[test]
    fn katanagent_thinking_false_configures_ollama_without_ui_thinking() {
        assert_eq!(
            KatanAgentOllamaAgent::ollama_thinking_option(Some("false")),
            Some("false".to_string())
        );
        assert_eq!(
            KatanAgentOllamaAgent::ollama_thinking_option(Some("default")),
            Some("false".to_string())
        );
        assert!(!KatanAgentOllamaAgent::thinking_enabled(Some("false")));
        assert!(!KatanAgentOllamaAgent::thinking_enabled(Some("default")));
        assert!(KatanAgentOllamaAgent::thinking_enabled(Some("low")));
    }

    #[test]
    fn ollama_model_catalog_honors_requested_model_without_fallback() -> Result<(), String> {
        let catalog = OllamaRuntimeCatalog::try_from_models(
            DEFAULT_OLLAMA_ENDPOINT.to_string(),
            vec!["gemma4:e4b".to_string(), "llama3".to_string()],
            Some("llama3".to_string()),
        )?;

        assert_eq!(
            catalog.model_options,
            vec!["ollama:llama3".to_string(), "ollama:gemma4:e4b".to_string()]
        );
        assert!(
            OllamaRuntimeCatalog::try_from_models(
                DEFAULT_OLLAMA_ENDPOINT.to_string(),
                vec!["gemma4:e4b".to_string()],
                Some("missing".to_string()),
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn sample_markdown_prompt_creates_tmp_file_candidate() {
        assert_eq!(
            AgentFileRequest::detect_target_path(
                "マークダウンの記法を取り入れたsampleを./tmpにsample.mdとして出力してください。"
            ),
            Some(SAMPLE_MARKDOWN_PATH)
        );
    }

    #[test]
    fn markdown_file_content_extracts_inner_fenced_block() {
        let content = "説明\n```markdown\n# sample\n\n- item\n```\n補足";

        assert_eq!(MarkdownFileContent::extract(content), "# sample\n\n- item");
    }

    #[test]
    fn mock_executor_emits_response_without_provider_command() {
        let (sender, receiver) = unbounded();

        ManualProviderExecutor::execute(test_job(7, "こんにちは"), sender);

        let events = receiver.try_iter().collect::<Vec<_>>();

        assert!(matches!(
            events.first(),
            Some(ManualProviderEvent::Chunk { .. })
        ));
        assert!(has_output_kind(&events, "file"));
        assert!(has_output_kind(&events, "diff"));
        assert!(has_output_kind(&events, "tool"));
        assert!(has_output_kind(&events, "permission"));
        assert!(matches!(
            events.last(),
            Some(ManualProviderEvent::Finished { .. })
        ));
    }

    #[test]
    fn mock_executor_emits_editing_outputs_before_finish() {
        let (sender, receiver) = unbounded();

        ManualProviderExecutor::execute(test_job(9, "編集して"), sender);
        let events = receiver.try_iter().collect::<Vec<_>>();

        assert!(has_output_kind(&events, "file"));
        assert!(has_output_kind(&events, "diff"));
        assert!(has_output_kind(&events, "tool"));
        assert!(has_output_kind(&events, "permission"));
        assert!(matches!(
            events.last(),
            Some(ManualProviderEvent::Finished { .. })
        ));
    }

    fn test_job(assistant_message_id: u64, prompt: &str) -> ManualProviderJob {
        ManualProviderJob {
            assistant_message_id,
            vendor_id: "claude-code".to_string(),
            execution: ManualProviderExecution::MockAgent,
            endpoint: None,
            model: Some("claude-sonnet-4-6".to_string()),
            thinking: Some("default".to_string()),
            permission: Some("auto".to_string()),
            prompt: prompt.to_string(),
            cwd: "/tmp/kcu".to_string(),
        }
    }

    fn has_output_kind(events: &[ManualProviderEvent], kind: &str) -> bool {
        events.iter().any(|event| match event {
            ManualProviderEvent::Output {
                kind: ChatOutputKind::FileCandidate(_),
                ..
            } => kind == "file",
            ManualProviderEvent::Output {
                kind: ChatOutputKind::DiffCandidate(_),
                ..
            } => kind == "diff",
            ManualProviderEvent::Output {
                kind: ChatOutputKind::ToolResult(_),
                ..
            } => kind == "tool",
            ManualProviderEvent::Output {
                kind: ChatOutputKind::PermissionRequest(_),
                ..
            } => kind == "permission",
            _ => false,
        })
    }
}
