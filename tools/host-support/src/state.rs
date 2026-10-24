use std::collections::BTreeSet;

use katana_chat_ui::{ChatSession, ChatUiHistorySessionSurface, ChatUiSurface, VendorUiState};

use crate::{
    ChatHistoryIssue, ChatHistoryRecord, ChatHistoryStore, ChatHostError, ChatOutputExecutor,
    PreparedHistorySave,
    surface::{history_entry, restrict_output_actions},
};

pub struct ChatHostState {
    pub(crate) session: ChatSession,
    pub(crate) history: ChatHistoryStore,
    pub(crate) output: ChatOutputExecutor,
    pub(crate) session_id: String,
    pub(crate) history_visible: bool,
    pub(crate) history_entries: Vec<ChatUiHistorySessionSurface>,
    history_issues: Vec<ChatHistoryIssue>,
    reported_history_issues: BTreeSet<String>,
}

pub struct ChatProviderPoll {
    pub state: VendorUiState,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatHostEffect {
    None,
    RequestAttachment,
    ProviderSelectionRequired { vendor_id: String },
    Submitted { message_id: u64 },
    OutputApplied { output_id: u64 },
    OutputReverted { output_id: u64 },
}

impl ChatHostState {
    pub fn new(
        mut session: ChatSession,
        history: ChatHistoryStore,
        output: ChatOutputExecutor,
    ) -> Result<Self, ChatHostError> {
        let (session_id, history_issues) = Self::restore_latest(&mut session, &history)?;
        Ok(Self {
            session,
            history,
            output,
            session_id,
            history_visible: false,
            history_entries: Vec::new(),
            history_issues,
            reported_history_issues: BTreeSet::new(),
        })
    }

    pub fn session(&self) -> &ChatSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut ChatSession {
        &mut self.session
    }

    pub fn surface(&self) -> ChatUiSurface {
        let mut surface = ChatUiSurface::from_render_model(&self.session.render_model());
        surface.history_panel.visible = self.history_visible;
        surface.history_panel.sessions = self.history_entries.clone();
        restrict_output_actions(&mut surface);
        surface
    }

    pub fn apply_provider_poll(&mut self, poll: ChatProviderPoll) {
        let provider_id = poll.state.active_vendor_id.clone();
        self.session.set_vendor_ui_state(poll.state);
        match poll.unavailable_reason {
            Some(reason) => self.session.set_provider_missing(reason),
            None if provider_id.is_empty() => self
                .session
                .set_provider_missing("provider is not configured"),
            None => self.session.set_provider_configured(provider_id),
        }
    }

    fn restore_latest(
        session: &mut ChatSession,
        history: &ChatHistoryStore,
    ) -> Result<(String, Vec<ChatHistoryIssue>), ChatHostError> {
        let (record, issues) = history
            .load_latest_with_diagnostics()
            .map_err(ChatHostError::History)?;
        let Some(record) = record else {
            return Ok((ChatHistoryStore::new_session_id(), issues));
        };
        session.restore_snapshot(record.snapshot);
        Ok((record.session_id, issues))
    }

    pub(crate) fn new_chat(&mut self) -> ChatHostEffect {
        let provider = self.session.render_model().provider;
        let vendor_state = self.session.vendor_ui_state().clone();
        self.session = ChatSession::new();
        match provider {
            katana_chat_ui::ProviderConnectionState::Configured(label) => {
                self.session.set_provider_configured(label);
            }
            katana_chat_ui::ProviderConnectionState::Missing(reason) => {
                self.session.set_provider_missing(reason);
            }
        }
        self.session.set_vendor_ui_state(vendor_state);
        self.session_id = ChatHistoryStore::new_session_id();
        self.history_visible = false;
        self.history_entries.clear();
        ChatHostEffect::None
    }

    pub(crate) fn toggle_history(&mut self) -> Result<ChatHostEffect, ChatHostError> {
        self.history_visible = !self.history_visible;
        if self.history_visible {
            self.refresh_history_entries()?;
        }
        Ok(ChatHostEffect::None)
    }

    pub(crate) fn select_history(
        &mut self,
        session_id: &str,
    ) -> Result<ChatHostEffect, ChatHostError> {
        let (record, issues) = self
            .history
            .load_with_diagnostics(session_id)
            .map_err(ChatHostError::History)?;
        self.remember_history_issues(issues);
        let record = record
            .ok_or_else(|| ChatHostError::History(format!("session is not found: {session_id}")))?;
        self.restore_record(record);
        Ok(ChatHostEffect::None)
    }

    pub(crate) fn submit(&mut self) -> Result<ChatHostEffect, ChatHostError> {
        if self.session.render_model().input.can_cancel {
            return Err(ChatHostError::Provider(
                "Stop the active provider before submitting another message".into(),
            ));
        }
        let message_id = self.session.submit_draft()?;
        Ok(ChatHostEffect::Submitted { message_id })
    }

    pub(crate) fn stop(&mut self) -> Result<ChatHostEffect, ChatHostError> {
        self.session.finish_assistant_message()?;
        Ok(ChatHostEffect::None)
    }

    fn refresh_history_entries(&mut self) -> Result<(), ChatHostError> {
        let listing = self
            .history
            .list_with_diagnostics()
            .map_err(ChatHostError::History)?;
        self.remember_history_issues(listing.issues);
        self.history_entries = listing.records.iter().map(history_entry).collect();
        Ok(())
    }

    fn remember_history_issues(&mut self, issues: Vec<ChatHistoryIssue>) {
        for issue in issues {
            if !self.history_issues.contains(&issue) {
                self.history_issues.push(issue);
            }
        }
    }

    pub(crate) fn remember_history_issues_for_boundary(&mut self, issues: Vec<ChatHistoryIssue>) {
        self.remember_history_issues(issues);
    }

    pub(crate) fn take_history_diagnostics(&mut self) -> Option<String> {
        let fresh = self
            .history_issues
            .iter()
            .filter_map(|issue| {
                let key = format!("{}: {}", issue.path.display(), issue.reason);
                self.reported_history_issues
                    .insert(key.clone())
                    .then_some(key)
            })
            .collect::<Vec<_>>();
        (!fresh.is_empty()).then(|| format!("history records skipped: {}", fresh.join("; ")))
    }

    fn restore_record(&mut self, record: ChatHistoryRecord) {
        self.session.restore_snapshot(record.snapshot);
        self.session_id = record.session_id;
        self.history_visible = false;
        self.history_entries.clear();
    }

    pub(crate) fn persist(&self) -> Result<(), ChatHostError> {
        self.prepare_persist()?
            .commit()
            .map_err(ChatHostError::History)
    }

    pub(crate) fn prepare_persist(&self) -> Result<PreparedHistorySave, ChatHostError> {
        self.prepare_session_persist(&self.session)
    }

    pub(crate) fn prepare_session_persist(
        &self,
        session: &ChatSession,
    ) -> Result<PreparedHistorySave, ChatHostError> {
        let provider_id = &session.vendor_ui_state().active_vendor_id;
        self.history
            .prepare_save(provider_id, &self.session_id, session.snapshot())
            .map_err(ChatHostError::History)
    }
}
