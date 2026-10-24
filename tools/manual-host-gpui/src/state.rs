use katana_chat_ui::{ChatUiSurface, kuc::ChatUiRuntimeEffect};
use kcu_host_support::{ManagedChatHost, ManagedHostCommand, ManagedHostUpdate};

use crate::trace::TraceWriter;

mod synchronization;

pub struct ManualGpuiState {
    worker: ManagedChatHost,
    surface: ChatUiSurface,
    latest_draft_seq: u64,
    last_error: Option<String>,
    trace: Option<TraceWriter>,
}

impl ManualGpuiState {
    pub fn new() -> Result<Self, String> {
        let root = std::env::current_dir().map_err(|error| error.to_string())?;
        let trace = TraceWriter::from_environment()?;
        let worker = ManagedChatHost::spawn(root).map_err(|error| error.to_string())?;
        Ok(Self {
            surface: worker.initial_surface(),
            worker,
            latest_draft_seq: 0,
            last_error: None,
            trace,
        })
    }

    pub fn surface(&self) -> ChatUiSurface {
        self.surface.clone()
    }

    pub fn apply_runtime_effect(&mut self, effect: ChatUiRuntimeEffect) {
        if let Err(error) = self.apply_effect(effect) {
            self.last_error = Some(error);
        }
    }

    pub fn take_error(&mut self) -> Option<String> {
        self.last_error.take()
    }

    pub fn poll(&mut self) -> Result<bool, String> {
        let mut changed = false;
        for _ in 0..32 {
            let Some(update) = self.worker.try_recv().map_err(|error| error.to_string())? else {
                break;
            };
            self.apply_update(update);
            changed = true;
        }
        Ok(changed)
    }

    fn apply_effect(&mut self, effect: ChatUiRuntimeEffect) -> Result<(), String> {
        match effect {
            ChatUiRuntimeEffect::DraftChanged(value) => {
                self.latest_draft_seq =
                    self.send(ManagedHostCommand::DraftChanged(value.clone()))?;
                self.surface.composer.text = value;
            }
            ChatUiRuntimeEffect::Action(action) => {
                let _ = self.send(ManagedHostCommand::Action(action))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn send(&self, command: ManagedHostCommand) -> Result<u64, String> {
        let seq = self
            .worker
            .send(command.clone())
            .map_err(|error| error.to_string())?;
        if let Some(trace) = &self.trace {
            trace.command("gpui", seq, &command);
        }
        Ok(seq)
    }

    fn apply_update(&mut self, mut update: ManagedHostUpdate) {
        if let Some(trace) = &self.trace {
            trace.update("gpui", &update);
        }
        synchronization::preserve_pending_draft(
            &mut update.surface,
            update.ack_seq,
            self.latest_draft_seq,
            &self.surface.composer.text,
        );
        self.surface = update.surface;
        if let Some(error) = update.error {
            self.last_error = Some(error);
        }
        if let Some(effect) = update.effect {
            self.record_host_effect(effect);
        }
    }

    fn record_host_effect(&mut self, effect: kcu_host_support::ChatHostEffect) {
        use kcu_host_support::ChatHostEffect;
        match effect {
            ChatHostEffect::None
            | ChatHostEffect::OutputApplied { .. }
            | ChatHostEffect::OutputReverted { .. } => {}
            other => self.last_error = Some(format!("Unprocessed host effect: {other:?}")),
        }
    }
}
