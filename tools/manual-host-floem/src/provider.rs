use std::{
    env,
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use crossbeam_channel::Sender;
use katana_acp_client::AiStreamEvent;
use katana_chat_ui::{
    ChatOutputKind, DiffCandidateOutput, FileCandidateOutput, ToolResultOutput, VendorUiState,
};
use serde::Deserialize;

use crate::ollama::ManualOllamaCatalog;

const OLLAMA_ID: &str = "ollama";
const CLAUDE_CODE_ID: &str = "claude-code";
const CODEX_CLI_ID: &str = "codex-cli";
const GITHUB_COPILOT_ID: &str = "github-copilot";
const OPENCODE_ID: &str = "opencode";
const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";
const OLLAMA_ENDPOINT_ENV: &str = "KCU_OLLAMA_ENDPOINT";

#[derive(Clone)]
pub(crate) struct ManualProviderRegistry {
    providers: Vec<ManualProviderDescriptor>,
}

impl ManualProviderRegistry {
    pub(crate) fn discover() -> Self {
        let mut providers = Vec::new();
        Self::push_ollama_document_backend(&mut providers);
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
    pub(crate) fn for_test_ollama_document() -> Self {
        Self {
            providers: vec![ManualProviderDescriptor::for_ollama_document_backend(
                DEFAULT_OLLAMA_ENDPOINT.to_string(),
                vec!["gemma4:e4b".to_string(), "llama3".to_string()],
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
            .map(|provider| provider.vendor_id.clone())
            .collect()
    }

    pub(crate) fn first_vendor_state(&self) -> Option<VendorUiState> {
        self.providers
            .first()
            .map(|provider| self.vendor_state(provider))
    }

    pub(crate) fn vendor_state_for(&self, vendor_id: &str) -> Option<VendorUiState> {
        self.providers
            .iter()
            .find(|provider| provider.vendor_id == vendor_id)
            .map(|provider| self.vendor_state(provider))
    }

    pub(crate) fn contains(&self, vendor_id: &str) -> bool {
        self.providers
            .iter()
            .any(|provider| provider.vendor_id == vendor_id)
    }

    pub(crate) fn execution_for(&self, vendor_id: &str) -> Option<ManualProviderExecution> {
        self.providers
            .iter()
            .find(|provider| provider.vendor_id == vendor_id)
            .map(|provider| provider.execution)
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

    fn push_ollama_document_backend(providers: &mut Vec<ManualProviderDescriptor>) {
        let endpoint = ollama_endpoint();
        let Ok(models) = ManualOllamaCatalog::list_model_names(endpoint.clone()) else {
            return;
        };
        providers.push(ManualProviderDescriptor::for_ollama_document_backend(
            endpoint, models,
        ));
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
}

impl ManualProviderDescriptor {
    fn from_detected_command(vendor_id: &str) -> Self {
        match vendor_id {
            CLAUDE_CODE_ID => Self::for_claude_code(),
            CODEX_CLI_ID => Self::for_codex_cli(),
            OPENCODE_ID => Self::for_opencode(),
            _ => Self::unknown(vendor_id),
        }
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
            execution: ManualProviderExecution::MockAgent,
        }
    }

    fn for_ollama_document_backend(endpoint: String, model_options: Vec<String>) -> Self {
        Self {
            vendor_id: OLLAMA_ID.to_string(),
            endpoint: Some(endpoint),
            model_options,
            thinking_options: vec![
                "false".to_string(),
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
            ],
            permission_options: Vec::new(),
            execution: ManualProviderExecution::OllamaDocument,
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
            execution: ManualProviderExecution::MockAgent,
        }
    }

    fn for_opencode() -> Self {
        Self {
            vendor_id: OPENCODE_ID.to_string(),
            endpoint: None,
            model_options: opencode_models(),
            thinking_options: vec!["false".to_string(), "true".to_string()],
            permission_options: Vec::new(),
            execution: ManualProviderExecution::MockAgent,
        }
    }

    fn unknown(vendor_id: &str) -> Self {
        Self {
            vendor_id: vendor_id.to_string(),
            endpoint: None,
            model_options: Vec::new(),
            thinking_options: Vec::new(),
            permission_options: Vec::new(),
            execution: ManualProviderExecution::MockAgent,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManualProviderExecution {
    MockAgent,
    OllamaDocument,
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
        toml::from_str(&content).unwrap_or_default()
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
            ManualProviderExecution::MockAgent => Self::execute_mock(job, sender),
            ManualProviderExecution::OllamaDocument => Self::execute_ollama_document(job, sender),
        }
    }

    fn execute_mock(job: ManualProviderJob, sender: Sender<ManualProviderEvent>) {
        if Self::emit_failure_if_needed(&job, &sender) {
            return;
        }
        Self::emit_thinking(&job, &sender);
        Self::emit_response(&job, &sender);
        Self::emit_outputs(&job, &sender);
        Self::emit_finished(job, &sender);
    }

    fn execute_ollama_document(job: ManualProviderJob, sender: Sender<ManualProviderEvent>) {
        let Some(endpoint) = job.endpoint.clone() else {
            Self::emit_failed(job, &sender, "Ollama endpoint is not configured");
            return;
        };
        let Some(model) = job.model.clone() else {
            Self::emit_failed(job, &sender, "Ollama model is not configured");
            return;
        };
        match OllamaDocumentExecutor::execute(&job, endpoint, model, &sender) {
            Ok(generated) => {
                Self::emit_ollama_document_output(&job, &generated, &sender);
                Self::emit_finished(job, &sender);
            }
            Err(error) => Self::emit_failed(job, &sender, &error),
        }
    }

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

    fn emit_response(job: &ManualProviderJob, sender: &Sender<ManualProviderEvent>) {
        let _ = sender.send(ManualProviderEvent::Chunk {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            content: ManualProviderMock::response(job),
        });
    }

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

    fn emit_failed(job: ManualProviderJob, sender: &Sender<ManualProviderEvent>, error: &str) {
        let _ = sender.send(ManualProviderEvent::Failed {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id,
            error: error.to_string(),
        });
    }

    fn emit_ollama_document_output(
        job: &ManualProviderJob,
        generated: &str,
        sender: &Sender<ManualProviderEvent>,
    ) {
        if generated.trim().is_empty() {
            return;
        }
        let _ = sender.send(ManualProviderEvent::Output {
            assistant_message_id: job.assistant_message_id,
            vendor_id: job.vendor_id.clone(),
            kind: ChatOutputKind::FileCandidate(FileCandidateOutput::new(
                format!("{}/tmp/kcu-ollama-document.md", job.cwd),
                "text/markdown",
                generated.to_string(),
            )),
        });
    }
}

struct OllamaDocumentExecutor;

impl OllamaDocumentExecutor {
    fn execute(
        job: &ManualProviderJob,
        endpoint: String,
        model: String,
        sender: &Sender<ManualProviderEvent>,
    ) -> Result<String, String> {
        let mut generated = String::new();
        ManualOllamaCatalog::execute_chat_streaming(
            endpoint,
            model,
            job.prompt.clone(),
            job.thinking.clone(),
            |event| Self::handle_event(job, sender, &mut generated, event),
        )?;
        Ok(generated)
    }

    fn handle_event(
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
        generated: &mut String,
        event: AiStreamEvent,
    ) -> Result<(), String> {
        match event {
            AiStreamEvent::Content(content) => {
                generated.push_str(&content);
                Self::send_chunk(job, sender, content)
            }
            AiStreamEvent::Thinking(content) => Self::send_thinking(job, sender, content),
        }
    }

    fn send_chunk(
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
        content: String,
    ) -> Result<(), String> {
        sender
            .send(ManualProviderEvent::Chunk {
                assistant_message_id: job.assistant_message_id,
                vendor_id: job.vendor_id.clone(),
                content,
            })
            .map_err(|error| error.to_string())
    }

    fn send_thinking(
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
        content: String,
    ) -> Result<(), String> {
        sender
            .send(ManualProviderEvent::ThinkingChunk {
                assistant_message_id: job.assistant_message_id,
                vendor_id: job.vendor_id.clone(),
                content,
            })
            .map_err(|error| error.to_string())
    }
}

struct ManualProviderMock;

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

fn ollama_endpoint() -> String {
    env::var(OLLAMA_ENDPOINT_ENV).unwrap_or_else(|_| DEFAULT_OLLAMA_ENDPOINT.to_string())
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
        ManualProviderEvent, ManualProviderExecution, ManualProviderExecutor, ManualProviderJob,
        ManualProviderRegistry, OllamaDocumentExecutor,
    };
    use crossbeam_channel::unbounded;
    use katana_acp_client::AiStreamEvent;
    use katana_chat_ui::ChatOutputKind;

    #[test]
    fn fixed_test_registry_excludes_ollama_direct_provider() {
        let registry = ManualProviderRegistry::for_test_agent();

        assert_eq!(
            registry.available_vendor_ids(),
            vec!["claude-code".to_string()]
        );
        assert!(!registry.contains("ollama"));
    }

    #[test]
    fn ollama_document_backend_is_default_low_cost_document_provider() {
        let registry = ManualProviderRegistry::for_test_ollama_document();
        let state = registry.first_vendor_state();

        assert!(state.is_some());
        if let Some(state) = state {
            assert_eq!(state.active_vendor_id, "ollama");
            assert_eq!(state.endpoint.as_deref(), Some("http://localhost:11434"));
            assert_eq!(state.selected_model.as_deref(), Some("gemma4:e4b"));
            assert_eq!(state.selected_thinking.as_deref(), Some("false"));
            assert!(state.selected_permission.is_none());
        }
        assert_eq!(
            registry.execution_for("ollama"),
            Some(ManualProviderExecution::OllamaDocument)
        );
    }

    #[test]
    fn mock_executor_emits_response_without_provider_command() {
        let (sender, receiver) = unbounded();

        ManualProviderExecutor::execute(test_job(7, "こんにちは"), sender);

        let first = receiver.try_recv();
        let second = receiver.try_recv();
        let third = receiver.try_recv();
        assert!(matches!(first, Ok(ManualProviderEvent::Chunk { .. })));
        assert!(matches!(second, Ok(ManualProviderEvent::Output { .. })));
        assert!(matches!(third, Ok(ManualProviderEvent::Output { .. })));
    }

    #[test]
    fn mock_executor_emits_editing_outputs_before_finish() {
        let (sender, receiver) = unbounded();

        ManualProviderExecutor::execute(test_job(9, "編集して"), sender);
        let events = receiver.try_iter().collect::<Vec<_>>();

        assert!(has_output_kind(&events, "file"));
        assert!(has_output_kind(&events, "diff"));
        assert!(has_output_kind(&events, "tool"));
        assert!(matches!(
            events.last(),
            Some(ManualProviderEvent::Finished { .. })
        ));
    }

    #[test]
    fn ollama_document_event_handler_streams_thinking_and_content() {
        let (sender, receiver) = unbounded();
        let mut generated = String::new();
        let job = ollama_job(11, "仕様書を作って");

        let thinking_result = OllamaDocumentExecutor::handle_event(
            &job,
            &sender,
            &mut generated,
            AiStreamEvent::Thinking("構成を整理しています".to_string()),
        );
        let content_result = OllamaDocumentExecutor::handle_event(
            &job,
            &sender,
            &mut generated,
            AiStreamEvent::Content("# 仕様書".to_string()),
        );

        let events = receiver.try_iter().collect::<Vec<_>>();
        assert!(thinking_result.is_ok());
        assert!(content_result.is_ok());
        assert_eq!(generated, "# 仕様書");
        assert!(matches!(
            events.first(),
            Some(ManualProviderEvent::ThinkingChunk { .. })
        ));
        assert!(matches!(
            events.get(1),
            Some(ManualProviderEvent::Chunk { .. })
        ));
    }

    #[test]
    fn ollama_document_output_is_file_candidate_only() {
        let (sender, receiver) = unbounded();
        let job = ollama_job(12, "README を作って");

        ManualProviderExecutor::emit_ollama_document_output(&job, "# README", &sender);

        let events = receiver.try_iter().collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert!(has_output_kind(&events, "file"));
        assert!(!has_output_kind(&events, "diff"));
        assert!(!has_output_kind(&events, "tool"));
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

    fn ollama_job(assistant_message_id: u64, prompt: &str) -> ManualProviderJob {
        ManualProviderJob {
            assistant_message_id,
            vendor_id: "ollama".to_string(),
            execution: ManualProviderExecution::OllamaDocument,
            endpoint: Some("http://localhost:11434".to_string()),
            model: Some("gemma4:e4b".to_string()),
            thinking: Some("false".to_string()),
            permission: None,
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
            _ => false,
        })
    }
}
