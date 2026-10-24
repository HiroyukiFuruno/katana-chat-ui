use katana_chat_ui::{ChatOutput, ChatUiAction, HostActionKind, OutputStatus, VendorUiState};

use crate::{ChatHostEffect, ChatHostError, ChatHostState};

impl ChatHostState {
    pub fn dispatch(&mut self, action: ChatUiAction) -> Result<ChatHostEffect, ChatHostError> {
        let output_action = matches!(&action, ChatUiAction::Output { .. });
        let effect = self.dispatch_inner(action)?;
        if !output_action {
            self.persist()?;
        }
        Ok(effect)
    }

    fn dispatch_inner(&mut self, action: ChatUiAction) -> Result<ChatHostEffect, ChatHostError> {
        match action {
            ChatUiAction::Attach => Ok(ChatHostEffect::RequestAttachment),
            ChatUiAction::RemoveAttachment { index } => self.remove_attachment(index),
            ChatUiAction::NewChat => Ok(self.new_chat()),
            ChatUiAction::ToggleHistory => self.toggle_history(),
            ChatUiAction::SelectHistory { session_id } => self.select_history(&session_id),
            ChatUiAction::Submit => self.submit(),
            ChatUiAction::Stop => self.stop(),
            ChatUiAction::SelectVendor { vendor_id } => self.select_vendor(vendor_id),
            ChatUiAction::SelectVendorControl { key, value } => {
                self.select_vendor_control(&key, value)
            }
            ChatUiAction::Output { output_id, kind } => self.output_action(output_id, kind),
            unsupported => Err(ChatHostError::UnsupportedAction(format!("{unsupported:?}"))),
        }
    }

    fn select_vendor(&self, vendor_id: String) -> Result<ChatHostEffect, ChatHostError> {
        if !self
            .session
            .vendor_ui_state()
            .available_vendor_ids
            .contains(&vendor_id)
        {
            return Err(ChatHostError::UnsupportedAction(format!(
                "unavailable provider: {vendor_id}"
            )));
        }
        Ok(ChatHostEffect::ProviderSelectionRequired { vendor_id })
    }

    fn remove_attachment(&mut self, index: usize) -> Result<ChatHostEffect, ChatHostError> {
        self.session
            .draft_mut()
            .remove_attachment(index)
            .ok_or_else(|| {
                ChatHostError::UnsupportedAction(format!("attachment index: {index}"))
            })?;
        Ok(ChatHostEffect::None)
    }

    fn select_vendor_control(
        &mut self,
        key: &str,
        value: String,
    ) -> Result<ChatHostEffect, ChatHostError> {
        let mut state = self.session.vendor_ui_state().clone();
        apply_vendor_control(&mut state, key, value)?;
        self.session.set_vendor_ui_state(state);
        Ok(ChatHostEffect::None)
    }

    fn output_action(
        &mut self,
        output_id: u64,
        kind: HostActionKind,
    ) -> Result<ChatHostEffect, ChatHostError> {
        let output = self.session.output(output_id)?.clone();
        if !output
            .host_actions()
            .iter()
            .any(|action| action.kind == kind)
        {
            return Err(ChatHostError::UnsupportedAction(format!(
                "output {output_id} does not allow {kind:?} in its current state"
            )));
        }
        match kind {
            HostActionKind::CreateFile | HostActionKind::ApplyDiff => {
                self.apply_output(output_id, &output)
            }
            HostActionKind::UndoChange => self.undo_output(output_id, &output),
            _ => Err(ChatHostError::UnsupportedAction(format!("output {kind:?}"))),
        }
    }

    fn apply_output(
        &mut self,
        output_id: u64,
        output: &ChatOutput,
    ) -> Result<ChatHostEffect, ChatHostError> {
        let mut next_session = self.session.clone();
        next_session.set_output_status(output_id, OutputStatus::Applied)?;
        let prepared = self.prepare_session_persist(&next_session)?;
        if let Err(reason) = self.output.apply(output) {
            return self.output_failure(output_id, reason);
        }
        if let Err(reason) = prepared.commit() {
            return Err(self.rollback_apply(output, reason));
        }
        self.session = next_session;
        Ok(ChatHostEffect::OutputApplied { output_id })
    }

    fn undo_output(
        &mut self,
        output_id: u64,
        output: &ChatOutput,
    ) -> Result<ChatHostEffect, ChatHostError> {
        let mut next_session = self.session.clone();
        next_session.set_output_status(output_id, OutputStatus::Reverted)?;
        let prepared = self.prepare_session_persist(&next_session)?;
        if let Err(reason) = self.output.undo(output) {
            return self.output_failure(output_id, reason);
        }
        if let Err(reason) = prepared.commit() {
            return Err(self.rollback_undo(output, reason));
        }
        self.session = next_session;
        Ok(ChatHostEffect::OutputReverted { output_id })
    }

    fn rollback_apply(&self, output: &ChatOutput, persist_reason: String) -> ChatHostError {
        let mut applied = output.clone();
        applied.status = OutputStatus::Applied;
        match self.output.undo(&applied) {
            Ok(_) => ChatHostError::History(persist_reason),
            Err(rollback_reason) => ChatHostError::History(format!(
                "{persist_reason}; output rollback failed: {rollback_reason}"
            )),
        }
    }

    fn rollback_undo(&self, output: &ChatOutput, persist_reason: String) -> ChatHostError {
        match self.output.apply(output) {
            Ok(_) => ChatHostError::History(persist_reason),
            Err(rollback_reason) => ChatHostError::History(format!(
                "{persist_reason}; output rollback failed: {rollback_reason}"
            )),
        }
    }

    fn output_failure(
        &mut self,
        output_id: u64,
        reason: String,
    ) -> Result<ChatHostEffect, ChatHostError> {
        self.session
            .set_output_status(output_id, OutputStatus::Failed(reason.clone()))?;
        self.persist()?;
        Err(ChatHostError::Output(reason))
    }
}

fn apply_vendor_control(
    state: &mut VendorUiState,
    key: &str,
    value: String,
) -> Result<(), ChatHostError> {
    let (options, selected) = match key {
        "model" => (&state.model_options, &mut state.selected_model),
        "mode" => (&state.mode_options, &mut state.selected_mode),
        "thinking" => (&state.thinking_options, &mut state.selected_thinking),
        "permission" => (&state.permission_options, &mut state.selected_permission),
        _ => {
            return Err(ChatHostError::UnsupportedAction(format!(
                "vendor control: {key}"
            )));
        }
    };
    if !options.contains(&value) {
        return Err(ChatHostError::UnsupportedAction(format!(
            "unavailable {key}: {value}"
        )));
    }
    *selected = Some(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use katana_chat_ui::{
        ChatOutput, ChatOutputKind, ChatSession, DiffCandidateOutput, FileCandidateOutput,
        VendorUiState,
    };

    use super::{ChatHostError, ChatHostState};
    use crate::{ChatHistoryStore, ChatOutputExecutor};

    #[test]
    fn rollback_undo_preserves_external_change_and_reports_both_errors()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("tmp"))?;
        fs::write(root.path().join("tmp/external.md"), "external")?;
        let kind = ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/external.md",
            "before",
            "after",
            "diff",
            "edit",
        ));
        let (host, output) = host_and_output(root.path(), kind)?;
        let error = host.rollback_undo(&output, "history commit failed".to_string());
        assert!(
            matches!(error, ChatHostError::History(reason) if reason.contains("history commit failed") && reason.contains("output rollback failed"))
        );
        assert_eq!(
            fs::read_to_string(root.path().join("tmp/external.md"))?,
            "external"
        );
        Ok(())
    }

    #[test]
    fn rollback_apply_reports_compensation_failure_without_creating_output()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        fs::create_dir_all(root.path().join("tmp"))?;
        fs::write(root.path().join("tmp/external.md"), "external")?;
        let kind = ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/external.md",
            "text/markdown",
            "generated",
        ));
        let (host, output) = host_and_output(root.path(), kind)?;
        let error = host.rollback_apply(&output, "history commit failed".to_string());
        assert!(
            matches!(error, ChatHostError::History(reason) if reason.contains("history commit failed") && reason.contains("output rollback failed"))
        );
        assert_eq!(
            fs::read_to_string(root.path().join("tmp/external.md"))?,
            "external"
        );
        Ok(())
    }

    fn host_and_output(
        root: &std::path::Path,
        kind: ChatOutputKind,
    ) -> Result<(ChatHostState, ChatOutput), Box<dyn std::error::Error>> {
        let mut session = ChatSession::new();
        session.set_provider_configured("KatanAgent");
        session.set_vendor_ui_state(VendorUiState::for_vendor("katanagent"));
        let message = session.start_assistant_stream("response")?;
        let output_id = session.add_output(message, kind)?;
        let output = session.output(output_id)?.clone();
        let host = ChatHostState::new(
            session,
            ChatHistoryStore::new(root.to_path_buf()),
            ChatOutputExecutor::new(root.to_path_buf()),
        )?;
        Ok((host, output))
    }
}
