use std::{
    env,
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use crossbeam_channel::Sender;
use katana_acp_client::AiStreamEvent;
use katana_chat_ui::VendorUiState;
use serde::Deserialize;

use crate::ollama::ManualOllamaCatalog;

const OLLAMA_ID: &str = "ollama";
const CLAUDE_CODE_ID: &str = "claude-code";
const CODEX_CLI_ID: &str = "codex-cli";
const GITHUB_COPILOT_ID: &str = "github-copilot";
const OPENCODE_ID: &str = "opencode";
const OLLAMA_ENDPOINT: &str = "http://localhost:11434";

#[derive(Clone)]
pub(crate) struct ManualProviderRegistry {
    providers: Vec<ManualProviderDescriptor>,
}

impl ManualProviderRegistry {
    pub(crate) fn discover() -> Self {
        let mut providers = Vec::new();
        Self::push_ollama(&mut providers);
        Self::push_command_provider(&mut providers, CLAUDE_CODE_ID, "claude");
        Self::push_command_provider(&mut providers, CODEX_CLI_ID, "codex");
        Self::push_github_copilot(&mut providers);
        Self::push_command_provider(&mut providers, OPENCODE_ID, "opencode");
        Self { providers }
    }

    #[cfg(test)]
    pub(crate) fn for_test_ollama() -> Self {
        Self {
            providers: vec![ManualProviderDescriptor {
                vendor_id: OLLAMA_ID.to_string(),
                endpoint: Some(OLLAMA_ENDPOINT.to_string()),
                model_options: vec!["test-model".to_string()],
                thinking_options: vec!["false".to_string()],
                permission_options: Vec::new(),
            }],
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

    fn push_ollama(providers: &mut Vec<ManualProviderDescriptor>) {
        let Ok(models) = ManualOllamaCatalog::list_model_names(OLLAMA_ENDPOINT) else {
            return;
        };
        providers.push(ManualProviderDescriptor {
            vendor_id: OLLAMA_ID.to_string(),
            endpoint: Some(OLLAMA_ENDPOINT.to_string()),
            model_options: models,
            thinking_options: vec![
                "false".to_string(),
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
            ],
            permission_options: Vec::new(),
        });
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
}

impl ManualProviderDescriptor {
    fn from_detected_command(vendor_id: &str) -> Self {
        match vendor_id {
            CLAUDE_CODE_ID => Self {
                vendor_id: vendor_id.to_string(),
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
            },
            CODEX_CLI_ID => Self::for_codex_cli(),
            OPENCODE_ID => Self::for_opencode(),
            _ => Self {
                vendor_id: vendor_id.to_string(),
                endpoint: None,
                model_options: Vec::new(),
                thinking_options: Vec::new(),
                permission_options: Vec::new(),
            },
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
        }
    }

    fn for_opencode() -> Self {
        Self {
            vendor_id: OPENCODE_ID.to_string(),
            endpoint: None,
            model_options: opencode_models(),
            thinking_options: vec!["false".to_string(), "true".to_string()],
            permission_options: Vec::new(),
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
        match toml::from_str(&content) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ManualProviderJob {
    pub(crate) assistant_message_id: u64,
    pub(crate) vendor_id: String,
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
        match Self::execute_job(&job, &sender) {
            Ok(()) => {
                let _ = sender.send(ManualProviderEvent::Finished {
                    assistant_message_id: job.assistant_message_id,
                    vendor_id: job.vendor_id,
                });
            }
            Err(error) => {
                let _ = sender.send(ManualProviderEvent::Failed {
                    assistant_message_id: job.assistant_message_id,
                    vendor_id: job.vendor_id,
                    error,
                });
            }
        }
    }

    fn execute_job(
        job: &ManualProviderJob,
        sender: &Sender<ManualProviderEvent>,
    ) -> Result<(), String> {
        match job.vendor_id.as_str() {
            OLLAMA_ID => execute_ollama(job, sender),
            CLAUDE_CODE_ID => execute_text_provider(job, sender, execute_claude_code),
            CODEX_CLI_ID => execute_text_provider(job, sender, execute_codex),
            GITHUB_COPILOT_ID => execute_text_provider(job, sender, execute_github_copilot),
            OPENCODE_ID => execute_text_provider(job, sender, execute_opencode),
            vendor_id => Err(format!("provider adapter is not registered: {vendor_id}")),
        }
    }
}

fn execute_ollama(
    job: &ManualProviderJob,
    sender: &Sender<ManualProviderEvent>,
) -> Result<(), String> {
    let endpoint = required(&job.endpoint, "endpoint")?;
    let model = required(&job.model, "model")?;
    let mut received = false;
    ManualOllamaCatalog::execute_chat_streaming(
        endpoint,
        model,
        job.prompt.clone(),
        job.thinking.clone(),
        |event| match event {
            AiStreamEvent::Content(content) => {
                received = true;
                send_chunk(job, sender, content)
            }
            AiStreamEvent::Thinking(content) => send_thinking_chunk(job, sender, content),
        },
    )?;
    if received {
        return Ok(());
    }
    Err("provider returned empty response".to_string())
}

fn execute_text_provider(
    job: &ManualProviderJob,
    sender: &Sender<ManualProviderEvent>,
    execute: fn(&ManualProviderJob) -> Result<String, String>,
) -> Result<(), String> {
    let content = execute(job)?;
    send_chunk(job, sender, content)
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

fn send_thinking_chunk(
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

fn execute_claude_code(job: &ManualProviderJob) -> Result<String, String> {
    let mut command = Command::new("claude");
    command
        .arg("-p")
        .arg("--output-format")
        .arg("text")
        .arg("--model")
        .arg(required(&job.model, "model")?);
    if let Some(thinking) = non_default(&job.thinking) {
        command.arg("--effort").arg(thinking);
    }
    if let Some(permission) = non_default(&job.permission) {
        command.arg("--permission-mode").arg(permission);
    }
    command.arg(&job.prompt);
    run_text_command(command)
}

fn execute_codex(job: &ManualProviderJob) -> Result<String, String> {
    let mut command = Command::new("codex");
    command
        .arg("exec")
        .arg("--cd")
        .arg(&job.cwd)
        .arg("--sandbox")
        .arg("read-only")
        .arg("--color")
        .arg("never");
    if let Some(model) = &job.model {
        command.arg("--model").arg(model);
    }
    if let Some(permission) = &job.permission {
        command.arg("--ask-for-approval").arg(permission);
    }
    command.arg(&job.prompt);
    run_text_command(command)
}

fn execute_github_copilot(job: &ManualProviderJob) -> Result<String, String> {
    let mut command = Command::new("gh");
    command.arg("copilot").arg("-p").arg(&job.prompt);
    run_text_command(command)
}

fn execute_opencode(job: &ManualProviderJob) -> Result<String, String> {
    let mut command = Command::new("opencode");
    command.arg("run").arg("--dir").arg(&job.cwd);
    if let Some(model) = &job.model {
        command.arg("--model").arg(model);
    }
    if job.thinking.as_deref() == Some("true") {
        command.arg("--thinking");
    }
    command.arg(&job.prompt);
    run_text_command(command)
}

fn required(value: &Option<String>, label: &str) -> Result<String, String> {
    match value {
        Some(value) => Ok(value.clone()),
        None => Err(format!("{label} is not configured")),
    }
}

fn non_default(value: &Option<String>) -> Option<String> {
    match value {
        Some(value) if value != "default" => Some(value.clone()),
        _ => None,
    }
}

fn run_text_command(mut command: Command) -> Result<String, String> {
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| error.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !output.status.success() {
        return Err(command_error(stdout, stderr));
    }
    if stdout.is_empty() {
        return Err("provider returned empty response".to_string());
    }
    Ok(stdout)
}

fn command_error(stdout: String, stderr: String) -> String {
    if !stderr.is_empty() {
        return stderr;
    }
    if !stdout.is_empty() {
        return stdout;
    }
    "provider command failed without output".to_string()
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
