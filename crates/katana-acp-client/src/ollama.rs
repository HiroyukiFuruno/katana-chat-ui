use crate::{AcpError, AiCapability, AiModel, AiProvider, AiRequest, AiResponse, ChatRole};
use std::time::Duration;

mod types;

use types::{OllamaChatRequest, OllamaChatResponse, OllamaMessage, OllamaTagsResponse};

const DEFAULT_ENDPOINT: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "llama3";
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: Option<String>, model: Option<String>) -> Result<Self, AcpError> {
        let endpoint = endpoint
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string())
            .trim_end_matches('/')
            .to_string();
        let model = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| AcpError::Transport(e.to_string()))?;

        Ok(Self {
            client,
            endpoint,
            model,
        })
    }

    fn build_context_message(request: &AiRequest) -> String {
        format!(
            "Document: {}\nContent:\n```\n{}\n```\nCursor Offset: {}\nDiagnostics: {:?}",
            request.context.uri,
            request.context.content,
            request.context.cursor_offset,
            request.context.diagnostics
        )
    }

    fn build_system_message(request: &AiRequest) -> String {
        let mut system_message = request
            .history
            .iter()
            .find(|turn| turn.role == ChatRole::System)
            .map(|turn| turn.content.clone())
            .unwrap_or_else(String::new);

        if !system_message.is_empty() {
            system_message.push_str("\n\n");
        }

        system_message.push_str("Current Document Context:\n");
        system_message.push_str(&Self::build_context_message(request));
        system_message.push_str("\n\nIntent: ");
        system_message.push_str(Self::intent_message(request));
        system_message
    }

    fn intent_message(request: &AiRequest) -> &'static str {
        match request.intent {
            crate::AiIntent::Modify => "Modify existing code.",
            crate::AiIntent::Create => "Create new code or documentation.",
            crate::AiIntent::Autofix => "Automatically fix errors or diagnostics.",
        }
    }

    fn build_messages(request: &AiRequest) -> Vec<OllamaMessage> {
        let mut messages = vec![OllamaMessage {
            role: "system".to_string(),
            content: Self::build_system_message(request),
        }];

        messages.extend(request.history.iter().filter_map(Self::history_message));
        messages.push(OllamaMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });
        messages
    }

    fn history_message(turn: &crate::ChatTurn) -> Option<OllamaMessage> {
        let role = match turn.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::System => return None,
        };

        Some(OllamaMessage {
            role: role.to_string(),
            content: turn.content.clone(),
        })
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    fn display_name(&self) -> &str {
        "Ollama"
    }

    async fn is_available(&self) -> bool {
        match self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    fn capabilities(&self) -> Vec<AiCapability> {
        vec![AiCapability {
            id: "chat".to_string(),
        }]
    }

    async fn list_models(&self) -> Result<Vec<AiModel>, AcpError> {
        let resp = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(|e| AcpError::Transport(e.to_string()))?
            .error_for_status()
            .map_err(|e| AcpError::Protocol(e.to_string()))?;

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| AcpError::Protocol(e.to_string()))?;

        Ok(tags
            .models
            .into_iter()
            .map(|m| AiModel {
                id: m.name.clone(),
                name: m.name,
            })
            .collect())
    }

    async fn execute(&self, request: &AiRequest) -> Result<AiResponse, AcpError> {
        let ollama_req = OllamaChatRequest {
            model: self.model.clone(),
            messages: Self::build_messages(request),
            stream: false,
        };

        let resp = self
            .client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| AcpError::Transport(e.to_string()))?
            .error_for_status()
            .map_err(|e| AcpError::Protocol(e.to_string()))?;

        let chat_resp: OllamaChatResponse = resp
            .json()
            .await
            .map_err(|e| AcpError::Protocol(e.to_string()))?;

        Ok(AiResponse {
            content: chat_resp.message.content,
        })
    }
}

#[cfg(test)]
mod tests;
