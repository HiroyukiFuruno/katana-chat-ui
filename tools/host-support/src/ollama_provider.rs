mod catalog;
mod state;
mod stream;

use catalog::{model_id, normalize};
use katana_acp_client::{AcpError, AiProvider, AiRequest, ollama::OllamaProvider};
use state::{PromptGuard, StreamState};
use std::sync::Arc;
use stream::stream_prompt;
use tokio::sync::Notify;

pub use catalog::KatanAgentOllamaModel;
pub use stream::KatanAgentOllamaEvent;

pub struct KatanAgentOllamaExecutor {
    endpoint: String,
    models: Vec<KatanAgentOllamaModel>,
    selected_model: Option<String>,
    stream_state: StreamState,
    cancellation: Arc<Notify>,
}

impl KatanAgentOllamaExecutor {
    pub fn new(endpoint: impl Into<String>) -> Result<Self, AcpError> {
        let endpoint = endpoint.into();
        if endpoint.trim().is_empty() {
            return Err(AcpError::Protocol(
                "Ollama endpoint is not configured".to_string(),
            ));
        }
        OllamaProvider::new(Some(endpoint.clone()), None)?;
        Ok(Self {
            endpoint,
            models: Vec::new(),
            selected_model: None,
            stream_state: StreamState::new(),
            cancellation: Arc::new(Notify::new()),
        })
    }

    #[must_use]
    pub fn vendor_label(&self) -> &'static str {
        "KatanAgent"
    }

    #[must_use]
    pub fn models(&self) -> &[KatanAgentOllamaModel] {
        &self.models
    }

    #[must_use]
    pub fn selected_model_id(&self) -> Option<&str> {
        self.selected_model.as_deref()
    }

    pub async fn refresh_model_catalog(&mut self) -> Result<Vec<KatanAgentOllamaModel>, AcpError> {
        let catalog = self.load_model_catalog().await;
        let models = match catalog {
            Ok(models) => models,
            Err(error) => {
                self.clear_catalog();
                return Err(error);
            }
        };
        self.models = models;
        self.clear_unavailable_selection();
        Ok(self.models.clone())
    }

    pub fn select_model_id(&mut self, id: &str) -> Result<(), AcpError> {
        self.selected_model = Some(model_id(&self.models, id)?);
        Ok(())
    }

    pub async fn prompt<OnEvent>(
        &self,
        request: &AiRequest,
        thinking: Option<String>,
        on_event: OnEvent,
    ) -> Result<(), AcpError>
    where
        OnEvent: FnMut(KatanAgentOllamaEvent) -> Result<(), AcpError>,
    {
        let provider = self.selected_provider()?;
        let _guard = self.begin_prompt()?;
        stream_prompt(
            &provider,
            request,
            thinking,
            &self.stream_state,
            &self.cancellation,
            on_event,
        )
        .await
    }

    pub fn cancel(&self) -> Result<(), AcpError> {
        self.stream_state.cancel()?;
        self.cancellation.notify_one();
        Ok(())
    }

    fn selected_provider(&self) -> Result<OllamaProvider, AcpError> {
        let Some(model) = self.selected_model.as_ref() else {
            return Err(AcpError::Protocol(
                "KatanAgent model is not selected".to_string(),
            ));
        };
        OllamaProvider::new(Some(self.endpoint.clone()), Some(model.clone()))
    }

    async fn load_model_catalog(&self) -> Result<Vec<KatanAgentOllamaModel>, AcpError> {
        let provider = OllamaProvider::new(Some(self.endpoint.clone()), None)?;
        normalize(provider.list_models().await?)
    }

    fn begin_prompt(&self) -> Result<PromptGuard, AcpError> {
        PromptGuard::begin(self.stream_state.clone())
    }

    fn clear_unavailable_selection(&mut self) {
        let Some(selected_model) = self.selected_model.as_deref() else {
            return;
        };
        if self.models.iter().all(|model| model.id != selected_model) {
            self.selected_model = None;
        }
    }

    fn clear_catalog(&mut self) {
        self.models.clear();
        self.selected_model = None;
    }
}
