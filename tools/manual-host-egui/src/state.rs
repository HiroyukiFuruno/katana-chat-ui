use std::path::PathBuf;

use katana_chat_ui::{ChatUiSurface, kuc::ChatUiAction};
use kcu_host_support::{ChatHostEffect, ManagedChatHost, ManagedHostCommand, ManagedHostUpdate};

use crate::font::ManualFontStatus;
use crate::trace::TraceWriter;

mod synchronization;

const MAX_PENDING_UPDATES: usize = 32;

pub struct ManualHostState {
    worker: ManagedChatHost,
    surface: ChatUiSurface,
    composer_text: String,
    sent_draft: String,
    latest_draft_seq: u64,
    last_error: Option<String>,
    trace: Option<TraceWriter>,
}

impl ManualHostState {
    pub fn new(_font_status: ManualFontStatus) -> Result<Self, String> {
        let root = std::env::current_dir().map_err(|error| error.to_string())?;
        Self::with_root(root)
    }

    pub fn surface(&self) -> ChatUiSurface {
        self.surface.clone()
    }

    pub fn composer_text_mut(&mut self) -> &mut String {
        &mut self.composer_text
    }

    pub fn dispatch(&mut self, action: ChatUiAction) -> Result<(), String> {
        self.flush_draft()?;
        self.send(ManagedHostCommand::Action(action)).map(|_| ())
    }

    pub fn flush_draft(&mut self) -> Result<(), String> {
        if self.composer_text == self.sent_draft {
            return Ok(());
        }
        let text = self.composer_text.clone();
        self.latest_draft_seq = self.send(ManagedHostCommand::DraftChanged(text.clone()))?;
        self.sent_draft = text;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<bool, String> {
        let mut changed = false;
        for _ in 0..MAX_PENDING_UPDATES {
            let Some(update) = self.worker.try_recv().map_err(|error| error.to_string())? else {
                break;
            };
            self.apply_update(update);
            changed = true;
        }
        Ok(changed)
    }

    pub fn record_error(&mut self, result: Result<(), String>) {
        if let Err(error) = result {
            self.last_error = Some(error);
        }
    }

    pub fn take_error(&mut self) -> Option<String> {
        self.last_error.take()
    }

    fn with_root(root: PathBuf) -> Result<Self, String> {
        let trace = TraceWriter::from_environment()?;
        let worker = ManagedChatHost::spawn(root).map_err(|error| error.to_string())?;
        let surface = worker.initial_surface();
        Ok(Self {
            composer_text: surface.composer.text.clone(),
            sent_draft: surface.composer.text.clone(),
            worker,
            surface,
            latest_draft_seq: 0,
            last_error: None,
            trace,
        })
    }

    fn send(&self, command: ManagedHostCommand) -> Result<u64, String> {
        let seq = self
            .worker
            .send(command.clone())
            .map_err(|error| error.to_string())?;
        if let Some(trace) = &self.trace {
            trace.command("egui", seq, &command);
        }
        Ok(seq)
    }

    fn apply_update(&mut self, mut update: ManagedHostUpdate) {
        if let Some(trace) = &self.trace {
            trace.update("egui", &update);
        }
        synchronization::preserve_pending_draft(
            &mut update.surface,
            update.ack_seq,
            self.latest_draft_seq,
            &self.composer_text,
        );
        self.accept_acknowledged_draft(&update);
        self.surface = update.surface;
        if let Some(error) = update.error {
            self.last_error = Some(error);
        }
        if let Some(effect) = update.effect {
            self.record_host_effect(effect);
        }
    }

    fn accept_acknowledged_draft(&mut self, update: &ManagedHostUpdate) {
        if update.ack_seq < self.latest_draft_seq {
            return;
        }
        self.composer_text = update.surface.composer.text.clone();
        self.sent_draft = self.composer_text.clone();
    }

    fn record_host_effect(&mut self, effect: ChatHostEffect) {
        match effect {
            ChatHostEffect::None
            | ChatHostEffect::OutputApplied { .. }
            | ChatHostEffect::OutputReverted { .. } => {}
            ChatHostEffect::RequestAttachment => {
                self.last_error = Some("Attachment picker is not available".to_string());
            }
            ChatHostEffect::ProviderSelectionRequired { vendor_id } => {
                self.last_error =
                    Some(format!("Provider connection is not available: {vendor_id}"));
            }
            ChatHostEffect::Submitted { .. } => {
                self.last_error = Some("No connected provider received the prompt".to_string());
            }
        }
    }
}

#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;
