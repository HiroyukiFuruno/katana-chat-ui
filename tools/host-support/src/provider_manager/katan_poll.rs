use katana_chat_ui::{AgentActivityKind, ChatAgentEvent};

use crate::{
    AcpProviderModel, ChatHostError, ChatHostState, KatanAgentOllamaEvent, from_ai_response,
};

use super::{
    KATANAGENT_ID, ProviderManager, ProviderPollEvent, ProviderPollKind, ProviderSlot, event,
    katanagent::KatanEvent,
};

impl ProviderManager {
    pub(super) fn handle_katan_event(
        &mut self,
        host: &mut ChatHostState,
        incoming: KatanEvent,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        match incoming {
            KatanEvent::CatalogReady { executor, models } => {
                self.apply_katan_catalog(host, executor, models, events)?
            }
            KatanEvent::CatalogFailed(reason) => {
                self.katan_slot_mut()?.catalog_failed();
                self.inventory.unavailable(KATANAGENT_ID, reason);
                self.sync_inventory(host);
                events.push(event(KATANAGENT_ID, ProviderPollKind::Unavailable));
            }
            KatanEvent::ModelSelected(model_id) => {
                self.apply_katan_model(host, model_id, events)?
            }
            KatanEvent::Stream {
                event: stream,
                structured_markdown,
            } => self.apply_katan_stream(host, stream, structured_markdown, events)?,
            KatanEvent::Complete(completion) => {
                self.complete_katan_stream(host, *completion, events)?
            }
        }
        Ok(())
    }

    fn apply_katan_catalog(
        &mut self,
        host: &mut ChatHostState,
        executor: crate::KatanAgentOllamaExecutor,
        models: Vec<crate::KatanAgentOllamaModel>,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        self.katan_slot_mut()?.catalog_ready(executor);
        let models = model_options(models);
        self.inventory.ready(KATANAGENT_ID, models.clone());
        if self.active_vendor_id.as_deref() == Some(KATANAGENT_ID) {
            self.select_katan_after_catalog(host, models)?;
        }
        self.set_restored_model_ready_state(host, KATANAGENT_ID);
        self.sync_inventory(host);
        events.push(event(KATANAGENT_ID, ProviderPollKind::Ready));
        Ok(())
    }

    fn select_katan_after_catalog(
        &mut self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        if self.should_preserve_restored_model(KATANAGENT_ID) {
            self.select_restored_katan(host, models)
        } else {
            self.select_katan(host, models)
        }
    }

    fn apply_katan_stream(
        &mut self,
        host: &mut ChatHostState,
        stream: KatanAgentOllamaEvent,
        structured_markdown: bool,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        if self.active_vendor_id.as_deref() != Some(KATANAGENT_ID)
            || self.katan_slot_mut()?.is_cancelling()
        {
            return Ok(());
        }
        match stream {
            KatanAgentOllamaEvent::Content(text) => {
                if !structured_markdown {
                    host.session_mut().append_assistant_chunk(text)?;
                }
                host.session_mut()
                    .set_assistant_activity(AgentActivityKind::Generating)?;
            }
            KatanAgentOllamaEvent::Thinking(text) => {
                host.session_mut()
                    .append_assistant_thinking("Thinking", text)?;
            }
        }
        events.push(event(KATANAGENT_ID, ProviderPollKind::AppliedToActive));
        Ok(())
    }

    fn apply_katan_model(
        &mut self,
        host: &mut ChatHostState,
        model_id: String,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        if self.active_vendor_id.as_deref() == Some(KATANAGENT_ID) {
            let mut vendor = host.session().vendor_ui_state().clone();
            vendor.selected_model = Some(model_id);
            host.session_mut().set_vendor_ui_state(vendor);
            events.push(event(KATANAGENT_ID, ProviderPollKind::AppliedToActive));
        }
        Ok(())
    }

    fn complete_katan_stream(
        &mut self,
        host: &mut ChatHostState,
        completion: super::katanagent::KatanCompletion,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        let slot_cancelled = self.katan_slot_mut()?.complete(completion.executor);
        let cancelled = completion.cancelled || slot_cancelled;
        if self.active_vendor_id.as_deref() != Some(KATANAGENT_ID) || cancelled {
            return Ok(());
        }
        match completion.result {
            Ok(()) => {
                self.complete_katan_markdown(host, &completion.request, &completion.response)?
            }
            Err(reason) => {
                host.session_mut().fail_assistant_message(reason)?;
                host.session_mut()
                    .set_provider_missing("KatanAgent Ollama prompt failed");
            }
        }
        events.push(event(KATANAGENT_ID, ProviderPollKind::AppliedToActive));
        Ok(())
    }

    fn complete_katan_markdown(
        &mut self,
        host: &mut ChatHostState,
        request: &katana_acp_client::AiRequest,
        response: &str,
    ) -> Result<(), ChatHostError> {
        match from_ai_response(request, response) {
            Ok(Some(kind)) => host
                .session_mut()
                .apply_agent_event(ChatAgentEvent::Output { kind })?,
            Ok(None) => {}
            Err(error) => {
                let reason = error.to_string();
                host.session_mut().fail_assistant_message(reason.clone())?;
                return Err(ChatHostError::Provider(reason));
            }
        }
        host.session_mut().finish_assistant_message()?;
        Ok(())
    }

    pub(super) fn katan_slot_mut(
        &mut self,
    ) -> Result<&mut super::katanagent::KatanSlot, ChatHostError> {
        match self.slots.get_mut(KATANAGENT_ID) {
            Some(ProviderSlot::Katan(slot)) => Ok(slot),
            _ => Err(ChatHostError::Provider(
                "KatanAgent slot is unavailable".into(),
            )),
        }
    }
}

fn model_options(models: Vec<crate::KatanAgentOllamaModel>) -> Vec<AcpProviderModel> {
    models
        .into_iter()
        .map(|model| AcpProviderModel {
            config_id: "ollama-model".into(),
            id: model.id,
            name: model.name,
            selected: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::model_options;
    use crate::KatanAgentOllamaModel;

    #[test]
    fn direct_catalog_models_do_not_select_a_model_implicitly() {
        let models = model_options(vec![KatanAgentOllamaModel {
            id: "llama3".into(),
            name: "Llama 3".into(),
        }]);
        assert_eq!(models[0].id, "llama3");
        assert!(!models[0].selected);
    }
}
