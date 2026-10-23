use katana_acp_client::AiRequest;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{KatanAgentOllamaExecutor, KatanAgentOllamaModel};

use super::{
    katan_request,
    katanagent::{KatanCompletion, KatanEvent},
};

pub(super) fn spawn_catalog(endpoint: String, events: UnboundedSender<KatanEvent>) {
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        let _ = events.send(KatanEvent::CatalogFailed(
            "Tokio runtime is not active".into(),
        ));
        return;
    };
    runtime.spawn(async move {
        let result = load_catalog(endpoint).await;
        let event = match result {
            Ok((executor, models)) => KatanEvent::CatalogReady { executor, models },
            Err(error) => KatanEvent::CatalogFailed(error),
        };
        let _ = events.send(event);
    });
}

async fn load_catalog(
    endpoint: String,
) -> Result<(KatanAgentOllamaExecutor, Vec<KatanAgentOllamaModel>), String> {
    let mut executor =
        KatanAgentOllamaExecutor::new(endpoint).map_err(|error| error.to_string())?;
    let models = executor
        .refresh_model_catalog()
        .await
        .map_err(|error| error.to_string())?;
    Ok((executor, models))
}

pub(super) fn spawn_prompt(
    executor: KatanAgentOllamaExecutor,
    request: AiRequest,
    events: UnboundedSender<KatanEvent>,
    mut cancellation: UnboundedReceiver<()>,
) {
    tokio::runtime::Handle::current().spawn(async move {
        let (result, cancelled, response) =
            run_prompt(&executor, &request, &events, &mut cancellation).await;
        let _ = events.send(KatanEvent::Complete(Box::new(KatanCompletion {
            executor,
            request,
            response,
            result: result.map_err(|error| error.to_string()),
            cancelled,
        })));
    });
}

async fn run_prompt(
    executor: &KatanAgentOllamaExecutor,
    request: &AiRequest,
    events: &UnboundedSender<KatanEvent>,
    cancellation: &mut UnboundedReceiver<()>,
) -> (Result<(), katana_acp_client::AcpError>, bool, String) {
    let events = events.clone();
    let structured_markdown = katan_request::is_markdown_request(request);
    let response = Arc::new(Mutex::new(String::new()));
    let collected = response.clone();
    let prompt = executor.prompt(request, None, move |event| {
        forward_event(event, &events, &collected, structured_markdown)
    });
    tokio::pin!(prompt);
    let (result, cancelled) = await_prompt(executor, prompt.as_mut(), cancellation).await;
    match collected_content(&response) {
        Ok(response) => (result, cancelled, response),
        Err(error) => (Err(error), cancelled, String::new()),
    }
}

fn forward_event(
    event: crate::KatanAgentOllamaEvent,
    events: &UnboundedSender<KatanEvent>,
    response: &Mutex<String>,
    structured_markdown: bool,
) -> Result<(), katana_acp_client::AcpError> {
    if let crate::KatanAgentOllamaEvent::Content(content) = &event {
        response
            .lock()
            .map_err(|_| katana_acp_client::AcpError::Protocol("response collector failed".into()))?
            .push_str(content);
    }
    events
        .send(KatanEvent::Stream {
            event,
            structured_markdown,
        })
        .map_err(|_| katana_acp_client::AcpError::Transport("KatanAgent poller ended".into()))
}

async fn await_prompt<F>(
    executor: &KatanAgentOllamaExecutor,
    mut prompt: Pin<&mut F>,
    cancellation: &mut UnboundedReceiver<()>,
) -> (Result<(), katana_acp_client::AcpError>, bool)
where
    F: Future<Output = Result<(), katana_acp_client::AcpError>>,
{
    tokio::select! {
        result = &mut prompt => (result, false),
        Some(()) = cancellation.recv() => (cancel_and_wait(executor, prompt).await, true),
    }
}

async fn cancel_and_wait<F>(
    executor: &KatanAgentOllamaExecutor,
    prompt: Pin<&mut F>,
) -> Result<(), katana_acp_client::AcpError>
where
    F: Future<Output = Result<(), katana_acp_client::AcpError>>,
{
    executor.cancel()?;
    prompt.await
}

fn collected_content(response: &Mutex<String>) -> Result<String, katana_acp_client::AcpError> {
    response
        .lock()
        .map(|content| content.clone())
        .map_err(|_| katana_acp_client::AcpError::Protocol("response collector failed".into()))
}
