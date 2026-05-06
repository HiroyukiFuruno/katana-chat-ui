use super::request::OllamaChatRequestBuilder;
use super::stream::OllamaStreamDecoder;
use super::types::{OllamaChatRequest, OllamaChatResponse, OllamaTagsResponse};
use crate::{AcpError, AiCapability, AiModel, AiProvider, AiRequest, AiResponse, AiStreamEvent};
use std::time::Duration;

const DEFAULT_ENDPOINT: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "llama3";
const REQUEST_TIMEOUT_SECS: u64 = 120;

pub struct OllamaProvider {
    client: reqwest::Client,
    pub(crate) endpoint: String,
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
            .map_err(|error| AcpError::Transport(error.to_string()))?;

        Ok(Self {
            client,
            endpoint,
            model,
        })
    }

    pub async fn execute_streaming<OnChunk>(
        &self,
        request: &AiRequest,
        mut on_chunk: OnChunk,
    ) -> Result<(), AcpError>
    where
        OnChunk: FnMut(String) -> Result<(), AcpError>,
    {
        self.execute_streaming_with_thinking(request, None, |event| {
            if let AiStreamEvent::Content(content) = event {
                return on_chunk(content);
            }
            Ok(())
        })
        .await
    }

    pub async fn execute_streaming_with_thinking<OnEvent>(
        &self,
        request: &AiRequest,
        thinking: Option<String>,
        mut on_event: OnEvent,
    ) -> Result<(), AcpError>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), AcpError>,
    {
        let ollama_req = Self::chat_request(&self.model, request, true, thinking);
        let mut response = self.send_chat_request(&ollama_req).await?;
        let mut pending = String::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| AcpError::Transport(error.to_string()))?
        {
            OllamaStreamDecoder::append_event(&mut pending, chunk.as_ref(), &mut on_event)?;
        }
        OllamaStreamDecoder::finish_event(&pending, &mut on_event)
    }

    async fn send_chat_request(
        &self,
        request: &OllamaChatRequest,
    ) -> Result<reqwest::Response, AcpError> {
        self.client
            .post(format!("{}/api/chat", self.endpoint))
            .json(request)
            .send()
            .await
            .map_err(|error| AcpError::Transport(error.to_string()))?
            .error_for_status()
            .map_err(|error| AcpError::Protocol(error.to_string()))
    }

    fn chat_request(
        model: &str,
        request: &AiRequest,
        stream: bool,
        thinking: Option<String>,
    ) -> OllamaChatRequest {
        OllamaChatRequestBuilder::build(
            model,
            request,
            Self::context_message(request),
            stream,
            thinking,
        )
    }

    fn context_message(request: &AiRequest) -> String {
        format!(
            "Document: {}\nContent:\n```\n{}\n```\nCursor Offset: {}\nDiagnostics: {:?}",
            request.context.uri,
            request.context.content,
            request.context.cursor_offset,
            request.context.diagnostics
        )
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
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn capabilities(&self) -> Vec<AiCapability> {
        vec![AiCapability {
            id: "chat".to_string(),
        }]
    }

    async fn list_models(&self) -> Result<Vec<AiModel>, AcpError> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(|error| AcpError::Transport(error.to_string()))?
            .error_for_status()
            .map_err(|error| AcpError::Protocol(error.to_string()))?;
        let tags: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|error| AcpError::Protocol(error.to_string()))?;

        Ok(tags
            .models
            .into_iter()
            .map(|model| AiModel {
                id: model.name.clone(),
                name: model.name,
            })
            .collect())
    }

    async fn execute(&self, request: &AiRequest) -> Result<AiResponse, AcpError> {
        let ollama_req = Self::chat_request(&self.model, request, false, None);
        let response = self.send_chat_request(&ollama_req).await?;
        let chat_resp: OllamaChatResponse = response
            .json()
            .await
            .map_err(|error| AcpError::Protocol(error.to_string()))?;

        Ok(AiResponse {
            content: chat_resp.message.content,
        })
    }
}
