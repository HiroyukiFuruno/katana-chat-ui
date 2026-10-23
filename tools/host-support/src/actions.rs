use katana_chat_ui::{ChatOutput, ChatUiAction, HostActionKind, OutputStatus, VendorUiState};

use crate::{ChatHostEffect, ChatHostError, ChatHostState};

impl ChatHostState {
    pub fn dispatch(&mut self, action: ChatUiAction) -> Result<ChatHostEffect, ChatHostError> {
        let effect = self.dispatch_inner(action)?;
        self.persist()?;
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
        if let Err(reason) = self.output.apply(output) {
            return self.output_failure(output_id, reason);
        }
        self.session
            .set_output_status(output_id, OutputStatus::Applied)?;
        Ok(ChatHostEffect::OutputApplied { output_id })
    }

    fn undo_output(
        &mut self,
        output_id: u64,
        output: &ChatOutput,
    ) -> Result<ChatHostEffect, ChatHostError> {
        if let Err(reason) = self.output.undo(output) {
            return self.output_failure(output_id, reason);
        }
        self.session
            .set_output_status(output_id, OutputStatus::Reverted)?;
        Ok(ChatHostEffect::OutputReverted { output_id })
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
