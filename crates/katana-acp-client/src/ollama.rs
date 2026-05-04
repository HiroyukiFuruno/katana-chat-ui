use crate::{AcpError, AiCapability, AiModel, AiProvider, AiRequest, AiResponse, ChatRole};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: Option<String>, model: Option<String>) -> Result<Self, AcpError> {
        let endpoint = endpoint
            .unwrap_or_else(|| "http://localhost:11434".to_string())
            .trim_end_matches('/')
            .to_string();
        let model = model.unwrap_or_else(|| "llama3".to_string());
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AcpError::Transport(e.to_string()))?;

        Ok(Self {
            client,
            endpoint,
            model,
        })
    }
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
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
        let resp: OllamaTagsResponse = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(|e| AcpError::Transport(e.to_string()))?
            .json()
            .await
            .map_err(|e| AcpError::Protocol(e.to_string()))?;

        Ok(resp
            .models
            .into_iter()
            .map(|m| AiModel {
                id: m.name.clone(),
                name: m.name,
            })
            .collect())
    }

    async fn execute(&self, request: &AiRequest) -> Result<AiResponse, AcpError> {
        let mut messages: Vec<OllamaMessage> = Vec::new();

        // Incorporate DocumentContext as a System message if not already present,
        // or augment the system message.
        let context_info = format!(
            "Document: {}\nContent:\n```\n{}\n```\nCursor Offset: {}\nDiagnostics: {:?}",
            request.context.uri,
            request.context.content,
            request.context.cursor_offset,
            request.context.diagnostics
        );

        let mut system_message = request
            .history
            .iter()
            .find(|t| t.role == ChatRole::System)
            .map(|t| t.content.clone())
            .unwrap_or_else(String::new);

        if !system_message.is_empty() {
            system_message.push_str("\n\n");
        }
        system_message.push_str("Current Document Context:\n");
        system_message.push_str(&context_info);

        messages.push(OllamaMessage {
            role: "system".to_string(),
            content: system_message,
        });

        for turn in &request.history {
            if turn.role == ChatRole::System {
                continue;
            }
            messages.push(OllamaMessage {
                role: match turn.role {
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                    ChatRole::System => unreachable!(),
                },
                content: turn.content.clone(),
            });
        }

        messages.push(OllamaMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        let ollama_req = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
        };

        let resp: OllamaChatResponse = self
            .client
            .post(format!("{}/api/chat", self.endpoint))
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| AcpError::Transport(e.to_string()))?
            .json()
            .await
            .map_err(|e| AcpError::Protocol(e.to_string()))?;

        Ok(AiResponse {
            content: resp.message.content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_normalization() {
        let p1 = OllamaProvider::new(None, None).unwrap();
        assert_eq!(p1.endpoint, "http://localhost:11434");

        let p2 = OllamaProvider::new(Some("http://127.0.0.1:11434/".to_string()), None).unwrap();
        assert_eq!(p2.endpoint, "http://127.0.0.1:11434");
    }
}
