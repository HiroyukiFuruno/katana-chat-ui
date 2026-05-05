//! katana-chat-ui: framework-neutral AI chat UI state.
//!
//! This crate owns chat state and a lightweight render model. Host applications
//! render that model with their own UI framework. Provider-specific affordances
//! are expressed through capabilities rather than host-specific widget code.
//!
//! Status: scaffolding. The full contract is tracked in the active OpenSpec
//! changes.

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

    pub fn render_model(&self) -> ChatPanelRenderModel {
        ChatPanelRenderModel {
            history_len: self.state.history.len(),
            draft: self.state.draft.clone(),
        }
    }

    pub fn draft_mut(&mut self) -> &mut String {
        &mut self.state.draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPanelRenderModel {
    pub history_len: usize,
    pub draft: String,
}

#[cfg(test)]
mod tests {
    use super::{ChatPanel, ChatPanelState};

    #[test]
    fn render_model_keeps_draft_without_ui_framework() {
        let mut state = ChatPanelState {
            draft: "hello".to_string(),
            ..ChatPanelState::default()
        };
        let panel = ChatPanel::new(&mut state);

        let model = panel.render_model();

        assert_eq!(model.history_len, 0);
        assert_eq!(model.draft, "hello");
    }
}
