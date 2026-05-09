use serde::{Deserialize, Serialize};

use super::ChatSession;
use crate::{
    AccountUsageSnapshot, ChatInputDraft, ChatMessage, ChatOutput, ContextUsageSnapshot,
    VendorUiState,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSessionSnapshot {
    pub version: u16,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub outputs: Vec<ChatOutput>,
    pub draft: ChatInputDraft,
    pub vendor_state: VendorUiState,
    pub context_usage: ContextUsageSnapshot,
    pub account_usage: AccountUsageSnapshot,
    pub next_message_id: u64,
    pub next_output_id: u64,
}

impl ChatSessionSnapshot {
    pub fn new(session: &ChatSession) -> Self {
        Self {
            version: 1,
            title: session.title.clone(),
            messages: session.messages.clone(),
            outputs: session.outputs.clone(),
            draft: session.draft.clone(),
            vendor_state: session.vendor_state.clone(),
            context_usage: session.context_usage.clone(),
            account_usage: session.account_usage.clone(),
            next_message_id: session.next_message_id,
            next_output_id: session.next_output_id,
        }
    }
}

impl ChatSession {
    pub fn snapshot(&self) -> ChatSessionSnapshot {
        ChatSessionSnapshot::new(self)
    }

    pub fn restore_snapshot(&mut self, snapshot: ChatSessionSnapshot) {
        self.title = snapshot.title;
        self.messages = snapshot.messages;
        self.outputs = snapshot.outputs;
        self.draft = snapshot.draft;
        self.vendor_state = snapshot.vendor_state;
        self.context_usage = snapshot.context_usage;
        self.account_usage = snapshot.account_usage;
        self.next_message_id = next_id(snapshot.next_message_id, self.max_message_id());
        self.next_output_id = next_id(snapshot.next_output_id, self.max_output_id());
    }

    fn max_message_id(&self) -> u64 {
        self.messages
            .iter()
            .map(|message| message.id)
            .max()
            .unwrap_or(0)
    }

    fn max_output_id(&self) -> u64 {
        self.outputs
            .iter()
            .map(|output| output.id)
            .max()
            .unwrap_or(0)
    }
}

fn next_id(stored_next_id: u64, max_existing_id: u64) -> u64 {
    stored_next_id.max(max_existing_id.saturating_add(1))
}
