use katana_acp_client::AiRequest;

use super::katan_worker::{spawn_catalog, spawn_prompt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
    ChatHostError, KatanAgentOllamaEvent, KatanAgentOllamaExecutor, KatanAgentOllamaModel,
};

pub(super) struct KatanSlot {
    state: KatanState,
    events: UnboundedReceiver<KatanEvent>,
    event_sender: UnboundedSender<KatanEvent>,
}

enum KatanState {
    CatalogLoading,
    Ready(KatanAgentOllamaExecutor),
    Streaming {
        cancel: UnboundedSender<()>,
        cancelled: bool,
        selected_model: Option<String>,
    },
    Unavailable,
}

pub(super) enum KatanEvent {
    CatalogReady {
        executor: KatanAgentOllamaExecutor,
        models: Vec<KatanAgentOllamaModel>,
    },
    CatalogFailed(String),
    ModelSelected(String),
    Stream {
        event: KatanAgentOllamaEvent,
        structured_markdown: bool,
    },
    Complete(Box<KatanCompletion>),
}

pub(super) struct KatanCompletion {
    pub(super) executor: KatanAgentOllamaExecutor,
    pub(super) request: AiRequest,
    pub(super) response: String,
    pub(super) result: Result<(), String>,
    pub(super) cancelled: bool,
}

impl KatanSlot {
    #[cfg(test)]
    pub(super) fn test_streaming_slot(
        executor: KatanAgentOllamaExecutor,
        cancelled: bool,
    ) -> (Self, UnboundedSender<KatanEvent>) {
        let (event_sender, events) = unbounded_channel();
        let (cancel, _cancellation) = unbounded_channel();
        let selected_model = executor.selected_model_id().map(str::to_owned);
        (
            Self {
                state: KatanState::Streaming {
                    cancel,
                    cancelled,
                    selected_model,
                },
                events,
                event_sender: event_sender.clone(),
            },
            event_sender,
        )
    }

    pub(super) fn selected_model_id(&self) -> Option<&str> {
        match &self.state {
            KatanState::Ready(executor) => executor.selected_model_id(),
            KatanState::Streaming { selected_model, .. } => selected_model.as_deref(),
            KatanState::CatalogLoading | KatanState::Unavailable => None,
        }
    }

    #[cfg(test)]
    pub(super) fn test_ready_slot(
        executor: KatanAgentOllamaExecutor,
    ) -> (Self, UnboundedSender<KatanEvent>) {
        let (event_sender, events) = unbounded_channel();
        (
            Self {
                state: KatanState::Ready(executor),
                events,
                event_sender: event_sender.clone(),
            },
            event_sender,
        )
    }

    #[cfg(test)]
    pub(super) fn test_slot() -> (Self, UnboundedSender<KatanEvent>) {
        let (event_sender, events) = unbounded_channel();
        (
            Self {
                state: KatanState::Unavailable,
                events,
                event_sender: event_sender.clone(),
            },
            event_sender,
        )
    }

    pub(super) fn probe(endpoint: String) -> Self {
        let (event_sender, events) = unbounded_channel();
        spawn_catalog(endpoint, event_sender.clone());
        Self {
            state: KatanState::CatalogLoading,
            events,
            event_sender,
        }
    }

    pub(super) fn try_next_event(&mut self) -> Option<KatanEvent> {
        self.events.try_recv().ok()
    }

    pub(super) fn catalog_ready(&mut self, executor: KatanAgentOllamaExecutor) {
        self.state = KatanState::Ready(executor);
    }

    pub(super) fn catalog_failed(&mut self) {
        self.state = KatanState::Unavailable;
    }

    pub(super) fn select_model(&mut self, id: &str) -> Result<(), ChatHostError> {
        let KatanState::Ready(executor) = &mut self.state else {
            return Err(ChatHostError::Provider("KatanAgent is not ready".into()));
        };
        executor
            .select_model_id(id)
            .map_err(|error| ChatHostError::Provider(error.to_string()))?;
        self.event_sender
            .send(KatanEvent::ModelSelected(id.to_string()))
            .map_err(|_| ChatHostError::Provider("KatanAgent poller ended".into()))
    }

    pub(super) fn start(&mut self, request: AiRequest) -> Result<(), ChatHostError> {
        let state = std::mem::replace(&mut self.state, KatanState::Unavailable);
        let KatanState::Ready(executor) = state else {
            self.state = state;
            return Err(ChatHostError::Provider("KatanAgent is not ready".into()));
        };
        let (cancel, cancellation) = unbounded_channel();
        let selected_model = executor.selected_model_id().map(str::to_owned);
        spawn_prompt(executor, request, self.event_sender.clone(), cancellation);
        self.state = KatanState::Streaming {
            cancel,
            cancelled: false,
            selected_model,
        };
        Ok(())
    }

    pub(super) fn can_prompt(&self) -> Result<(), ChatHostError> {
        let KatanState::Ready(executor) = &self.state else {
            return Err(ChatHostError::Provider("KatanAgent is not ready".into()));
        };
        if executor.selected_model_id().is_none() {
            return Err(ChatHostError::Provider(
                "KatanAgent model is not selected".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn stop(&mut self) -> Result<(), ChatHostError> {
        let KatanState::Streaming {
            cancel, cancelled, ..
        } = &mut self.state
        else {
            return Err(ChatHostError::Provider(
                "KatanAgent has no active Ollama prompt".into(),
            ));
        };
        cancel
            .send(())
            .map_err(|_| ChatHostError::Provider("KatanAgent prompt worker ended".into()))?;
        *cancelled = true;
        Ok(())
    }

    pub(super) fn complete(&mut self, executor: KatanAgentOllamaExecutor) -> bool {
        let cancelled = matches!(
            self.state,
            KatanState::Streaming {
                cancelled: true,
                ..
            }
        );
        self.state = KatanState::Ready(executor);
        cancelled
    }

    pub(super) fn is_cancelling(&self) -> bool {
        matches!(
            self.state,
            KatanState::Streaming {
                cancelled: true,
                ..
            }
        )
    }
}
