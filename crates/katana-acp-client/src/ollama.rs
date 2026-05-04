use crate::{AcpError, AiCapability, AiModel, AiProvider, AiRequest, AiResponse, ChatRole};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: Option<String>, model: Option<String>) -> Self {
        let endpoint = endpoint
            .unwrap_or_else(|| "http://localhost:11434".to_string())
            .trim_end_matches('/')
            .to_string();
        let model = model.unwrap_or_else(|| "llama3".to_string());
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            endpoint,
            model,
        }
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
        self.client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .is_ok()
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
        let mut messages: Vec<OllamaMessage> = request
            .history
            .iter()
            .map(|t| OllamaMessage {
                role: match t.role {
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                    ChatRole::System => "system".to_string(),
                },
                content: t.content.clone(),
            })
            .collect();

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
        let p1 = OllamaProvider::new(None, None);
        assert_eq!(p1.endpoint, "http://localhost:11434");

        let p2 = OllamaProvider::new(Some("http://127.0.0.1:11434/".to_string()), None);
        assert_eq!(p2.endpoint, "http://127.0.0.1:11434");
    }
}
