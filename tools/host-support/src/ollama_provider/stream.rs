use katana_acp_client::{AcpError, AiRequest, AiStreamEvent, ollama::OllamaProvider};
use tokio::sync::Notify;

use super::state::StreamState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KatanAgentOllamaEvent {
    Content(String),
    Thinking(String),
}

pub(super) async fn stream_prompt<OnEvent>(
    provider: &OllamaProvider,
    request: &AiRequest,
    thinking: Option<String>,
    state: &StreamState,
    cancellation: &Notify,
    on_event: OnEvent,
) -> Result<(), AcpError>
where
    OnEvent: FnMut(KatanAgentOllamaEvent) -> Result<(), AcpError>,
{
    tokio::select! {
        biased;
        () = wait_for_cancellation(cancellation, state) => Err(cancelled_error()),
        result = stream_response(provider, request, thinking, on_event) => result,
    }
}

async fn stream_response<OnEvent>(
    provider: &OllamaProvider,
    request: &AiRequest,
    thinking: Option<String>,
    mut on_event: OnEvent,
) -> Result<(), AcpError>
where
    OnEvent: FnMut(KatanAgentOllamaEvent) -> Result<(), AcpError>,
{
    let mut has_content = false;
    provider
        .execute_streaming_with_thinking(request, thinking, |event| {
            has_content |=
                matches!(&event, AiStreamEvent::Content(content) if !content.trim().is_empty());
            on_event(event.into())
        })
        .await?;
    if has_content {
        Ok(())
    } else {
        Err(AcpError::Protocol("Ollama response was empty".to_string()))
    }
}

async fn wait_for_cancellation(cancellation: &Notify, state: &StreamState) {
    loop {
        cancellation.notified().await;
        if state.is_cancelling() {
            return;
        }
    }
}

fn cancelled_error() -> AcpError {
    AcpError::Transport("KatanAgent Ollama prompt was cancelled".to_string())
}

impl From<AiStreamEvent> for KatanAgentOllamaEvent {
    fn from(event: AiStreamEvent) -> Self {
        match event {
            AiStreamEvent::Content(content) => Self::Content(content),
            AiStreamEvent::Thinking(content) => Self::Thinking(content),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StreamState, wait_for_cancellation};
    use crate::ollama_provider::state::PromptGuard;
    use std::{
        future::Future,
        pin::pin,
        task::{Context, Poll, Waker},
    };
    use tokio::sync::Notify;

    #[test]
    fn cancellation_notification_keeps_a_permit_before_polling() {
        let cancellation = Notify::new();
        let notification = cancellation.notified();
        let mut notification = pin!(notification);
        cancellation.notify_one();
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            notification.as_mut().poll(&mut context),
            Poll::Ready(())
        ));
    }

    #[test]
    fn consumed_cancellation_permit_does_not_cancel_the_next_prompt() {
        let cancellation = Notify::new();
        let first = cancellation.notified();
        let mut first = pin!(first);
        cancellation.notify_one();
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(first.as_mut().poll(&mut context), Poll::Ready(())));
        let second = cancellation.notified();
        let mut second = pin!(second);
        assert!(matches!(second.as_mut().poll(&mut context), Poll::Pending));
    }

    #[test]
    fn dropped_cancelled_prompt_does_not_cancel_the_next_prompt()
    -> Result<(), katana_acp_client::AcpError> {
        let state = StreamState::new();
        let cancellation = Notify::new();
        let guard = PromptGuard::begin(state.clone())?;
        {
            let first_waiter = wait_for_cancellation(&cancellation, &state);
            let mut first_waiter = pin!(first_waiter);
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(
                first_waiter.as_mut().poll(&mut context),
                Poll::Pending
            ));
            state.cancel()?;
            cancellation.notify_one();
        }
        drop(guard);
        let next_guard = PromptGuard::begin(state.clone())?;
        let next_waiter = wait_for_cancellation(&cancellation, &state);
        let mut next_waiter = pin!(next_waiter);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            next_waiter.as_mut().poll(&mut context),
            Poll::Pending
        ));
        drop(next_guard);
        Ok(())
    }
}
