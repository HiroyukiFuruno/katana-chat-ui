use katana_acp_client::ollama::OllamaProvider;
use katana_acp_client::{AiIntent, AiProvider, AiRequest, DocumentContext};
use katana_acp_client::{AcpError, AiStreamEvent};

pub(crate) struct ManualOllamaCatalog;

impl ManualOllamaCatalog {
    pub(crate) fn list_model_names(endpoint: impl Into<String>) -> Result<Vec<String>, String> {
        let provider =
            OllamaProvider::new(Some(endpoint.into()), None).map_err(|error| error.to_string())?;
        let runtime = Self::runtime()?;
        let models = runtime
            .block_on(provider.list_models())
            .map_err(|error| error.to_string())?;
        let names = models.into_iter().map(|it| it.name).collect::<Vec<_>>();
        if names.is_empty() {
            return Err("Ollama /api/tags returned no models".to_string());
        }
        Ok(names)
    }

    pub(crate) fn execute_chat_streaming<OnEvent>(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        prompt: impl Into<String>,
        thinking: Option<String>,
        mut on_event: OnEvent,
    ) -> Result<(), String>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), String>,
    {
        let provider = OllamaProvider::new(Some(endpoint.into()), Some(model.into()))
            .map_err(|error| error.to_string())?;
        let request = Self::request(prompt.into());
        let runtime = Self::runtime()?;
        runtime
            .block_on(provider.execute_streaming_with_thinking(&request, thinking, |event| {
                on_event(event).map_err(AcpError::Transport)
            }))
            .map_err(|error| error.to_string())
    }

    fn request(prompt: String) -> AiRequest {
        AiRequest {
            intent: AiIntent::Create,
            context: DocumentContext {
                uri: "manual-host://floem".to_string(),
                content: String::new(),
                cursor_offset: 0,
                diagnostics: Vec::new(),
            },
            prompt,
            history: Vec::new(),
        }
    }

    fn runtime() -> Result<tokio::runtime::Runtime, String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())
    }
}
