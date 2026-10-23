use katana_acp_client::AcpError;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

pub(super) const CANCELLING: u8 = 2;
const IDLE: u8 = 0;
const STREAMING: u8 = 1;

#[derive(Clone)]
pub(super) struct StreamState(Arc<AtomicU8>);

impl StreamState {
    pub(super) fn new() -> Self {
        Self(Arc::new(AtomicU8::new(IDLE)))
    }

    pub(super) fn cancel(&self) -> Result<(), AcpError> {
        self.0
            .compare_exchange(STREAMING, CANCELLING, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                AcpError::Protocol("KatanAgent has no active Ollama prompt".to_string())
            })?;
        Ok(())
    }

    pub(super) fn is_cancelling(&self) -> bool {
        self.0.load(Ordering::Acquire) == CANCELLING
    }

    fn begin(&self) -> Result<(), AcpError> {
        self.0
            .compare_exchange(IDLE, STREAMING, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| {
                AcpError::Protocol("KatanAgent Ollama prompt is already active".to_string())
            })
    }

    fn finish(&self) {
        self.0.store(IDLE, Ordering::Release);
    }
}

pub(super) struct PromptGuard {
    state: StreamState,
}

impl PromptGuard {
    pub(super) fn begin(state: StreamState) -> Result<Self, AcpError> {
        state.begin()?;
        Ok(Self { state })
    }
}

impl Drop for PromptGuard {
    fn drop(&mut self) {
        self.state.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::{PromptGuard, StreamState};

    #[test]
    fn dropped_prompt_guard_releases_stream_state() -> Result<(), katana_acp_client::AcpError> {
        let state = StreamState::new();
        let guard = PromptGuard::begin(state.clone())?;
        state.cancel()?;
        assert!(PromptGuard::begin(state.clone()).is_err());
        drop(guard);
        let next = PromptGuard::begin(state)?;
        drop(next);
        Ok(())
    }
}
