use serde::{Deserialize, Serialize};

use crate::ChatOutputKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatAgentRunConfig {
    pub provider_id: String,
    pub runtime: ChatAgentRuntime,
    pub model: String,
    pub thinking: ChatAgentThinking,
    pub permission_mode: String,
    pub working_directory: String,
}

impl ChatAgentRunConfig {
    pub fn new(
        provider_id: impl Into<String>,
        runtime: ChatAgentRuntime,
        model: impl Into<String>,
        thinking: ChatAgentThinking,
        permission_mode: impl Into<String>,
        working_directory: impl Into<String>,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            runtime,
            model: model.into(),
            thinking,
            permission_mode: permission_mode.into(),
            working_directory: working_directory.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatAgentRuntime {
    AcpProcess(ChatAgentProcessRuntime),
    ExternalProcess(ChatAgentProcessRuntime),
    Mock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatAgentProcessRuntime {
    pub command: String,
    pub args: Vec<String>,
}

impl ChatAgentProcessRuntime {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatAgentThinking {
    Disabled,
    Enabled { level: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatAgentEvent {
    Chunk { content: String },
    ThinkingChunk { label: String, content: String },
    Output { kind: ChatOutputKind },
    Complete,
    Failed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::{ChatAgentProcessRuntime, ChatAgentRunConfig, ChatAgentRuntime, ChatAgentThinking};

    #[test]
    fn katanagent_config_keeps_ollama_as_runtime_argument() {
        let config = ChatAgentRunConfig::new(
            "katanagent",
            ChatAgentRuntime::AcpProcess(ChatAgentProcessRuntime::new(
                "katanagent",
                vec!["--runtime".to_string(), "ollama".to_string()],
            )),
            "qwen2.5-coder:7b",
            ChatAgentThinking::Enabled {
                level: "low".to_string(),
            },
            "ask",
            "/workspace",
        );

        assert_eq!(config.provider_id, "katanagent");
        assert_ne!(config.provider_id, "ollama");
        assert!(matches!(config.runtime, ChatAgentRuntime::AcpProcess(_)));
        if let ChatAgentRuntime::AcpProcess(runtime) = config.runtime {
            assert_eq!(runtime.command, "katanagent");
            assert!(runtime.args.iter().any(|it| it == "ollama"));
        }
    }
}
