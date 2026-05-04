//! katana-chat-ui: vendor-neutral egui chat side-panel widget.
//!
//! This crate ships a reusable chat UI that talks to any ACP-compatible agent
//! through `katana-acp-client`. KatanA hosts the widget but does not implement
//! per-vendor UI variations: vendor-specific affordances are negotiated through
//! ACP capabilities and rendered by this widget.
//!
//! Status: scaffolding. The widget API and state model are added during the
//! v0.22.14 change.

use katana_acp_client::ChatTurn;

#[derive(Debug, Default)]
pub struct ChatPanelState {
    pub history: Vec<ChatTurn>,
    pub draft: String,
}

pub struct ChatPanel<'a> {
    state: &'a mut ChatPanelState,
}

impl<'a> ChatPanel<'a> {
    pub fn new(state: &'a mut ChatPanelState) -> Self {
        Self { state }
    }

    pub fn ui(self, ui: &mut egui::Ui) {
        ui.label("[scaffold] katana-chat-ui");
        ui.separator();
        for turn in &self.state.history {
            ui.label(format!("{:?}: {}", turn.role, turn.content));
        }
        ui.text_edit_multiline(&mut self.state.draft);
    }
}
