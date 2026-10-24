use katana_chat_ui::{VendorUiOption, VendorUiState};

use crate::{AcpProviderModel, ChatHostError, ChatHostState};

use super::{KATANAGENT_ID, ProviderManager, ProviderSlot, katan_request};

impl ProviderManager {
    pub(super) fn select_katan(
        &mut self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        let selected_model = self
            .slots
            .get(KATANAGENT_ID)
            .and_then(|slot| match slot {
                ProviderSlot::Katan(slot) => slot.selected_model_id(),
                ProviderSlot::Acp(_) => None,
            })
            .map(str::to_owned);
        self.configure_katan(host, models, selected_model)
    }

    pub(super) fn select_restored_katan(
        &mut self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        let restored_model = (host.session().vendor_ui_state().active_vendor_id == KATANAGENT_ID)
            .then(|| host.session().vendor_ui_state().selected_model.clone())
            .flatten();
        self.configure_katan(host, models, restored_model)
    }

    fn configure_katan(
        &self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
        restored_model: Option<String>,
    ) -> Result<(), ChatHostError> {
        let mut vendor = VendorUiState::for_vendor(KATANAGENT_ID);
        vendor.model_options = models.iter().map(|model| model.id.clone()).collect();
        vendor.model_choices = models
            .iter()
            .map(|model| VendorUiOption {
                id: model.id.clone(),
                label: model.name.clone(),
            })
            .collect();
        vendor.selected_model = restored_model;
        host.session_mut().set_vendor_ui_state(vendor);
        host.session_mut().set_provider_configured(KATANAGENT_ID);
        Ok(())
    }

    pub(super) async fn submit_katan(
        &mut self,
        host: &mut ChatHostState,
    ) -> Result<(), ChatHostError> {
        self.katan_slot_mut()?.can_prompt()?;
        let request =
            katan_request::request(host, &self.workspace_root).map_err(ChatHostError::Provider)?;
        host.submit()?;
        host.session_mut().start_assistant_stream(String::new())?;
        self.katan_slot_mut()?.start(request)?;
        host.persist()
    }

    pub(super) async fn stop_katan(
        &mut self,
        host: &mut ChatHostState,
    ) -> Result<(), ChatHostError> {
        self.katan_slot_mut()?.stop()?;
        host.session_mut().finish_assistant_message()?;
        host.persist()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{KATANAGENT_ID, ProviderManager};
    use crate::{
        AcpProviderModel, ChatHistoryStore, ChatHostState, ChatOutputExecutor,
        KatanAgentOllamaExecutor, KatanAgentOllamaModel,
        provider_manager::{ProviderSlot, katanagent::KatanSlot},
    };
    use katana_chat_ui::{ChatSession, VendorUiState};

    #[test]
    fn ordinary_catalog_selection_does_not_retain_a_restored_model()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut session = ChatSession::new();
        session.set_vendor_ui_state(
            VendorUiState::for_vendor(KATANAGENT_ID).with_models(vec!["saved".into()], "saved"),
        );
        let mut host = ChatHostState::new(
            session,
            ChatHistoryStore::new(root.path().into()),
            ChatOutputExecutor::new(root.path().into()),
        )?;

        let mut manager = ProviderManager::new();
        manager.select_katan(&mut host, vec![model()])?;

        assert_eq!(host.session().vendor_ui_state().selected_model, None);
        Ok(())
    }

    #[test]
    fn reselecting_katan_preserves_executor_model_in_ui_and_history()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut manager = manager_with_confirmed_model()?;
        let mut chat_host = host(root.path())?;
        manager.select(&mut chat_host, KATANAGENT_ID)?;
        assert_selected(&chat_host, Some("confirmed"));
        manager.active_vendor_id = Some("codex-cli".into());
        manager.select(&mut chat_host, KATANAGENT_ID)?;
        assert_selected(&chat_host, Some("confirmed"));
        let restored = host(root.path())?;
        assert_selected(&restored, Some("confirmed"));
        Ok(())
    }

    #[test]
    fn same_provider_reselect_preserves_streaming_model() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = tempfile::tempdir()?;
        let mut manager = manager_with_slot(streaming_slot(false)?);
        let mut host = host(root.path())?;
        host.session_mut().set_provider_configured(KATANAGENT_ID);
        host.session_mut().start_assistant_stream("")?;
        manager.active_vendor_id = Some(KATANAGENT_ID.into());
        manager.select(&mut host, KATANAGENT_ID)?;
        assert_selected(&host, Some("confirmed"));
        assert!(host.session().render_model().input.can_cancel);
        Ok(())
    }

    #[test]
    fn same_provider_reselect_preserves_model_while_cancellation_completes()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut manager = manager_with_slot(streaming_slot(true)?);
        let mut chat_host = host(root.path())?;
        manager.active_vendor_id = Some(KATANAGENT_ID.into());
        manager.select(&mut chat_host, KATANAGENT_ID)?;
        assert_selected(&chat_host, Some("confirmed"));
        assert!(manager.katan_slot_mut()?.is_cancelling());
        let restored = host(root.path())?;
        assert_selected(&restored, Some("confirmed"));
        Ok(())
    }

    #[test]
    fn cleared_executor_model_is_not_restored_from_saved_ui()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut manager = manager_with_confirmed_model()?;
        let executor = KatanAgentOllamaExecutor::test_with_models(vec![KatanAgentOllamaModel {
            id: "other".into(),
            name: "Other".into(),
        }])?;
        manager.katan_slot_mut()?.catalog_ready(executor);
        manager.inventory.ready(KATANAGENT_ID, vec![other_model()]);
        manager.active_vendor_id = Some("codex-cli".into());
        let mut host = host(root.path())?;
        host.session_mut().set_vendor_ui_state(
            VendorUiState::for_vendor(KATANAGENT_ID)
                .with_models(vec!["confirmed".into()], "confirmed"),
        );
        manager.select(&mut host, KATANAGENT_ID)?;
        assert_selected(&host, None);
        assert!(manager.katan_slot_mut()?.selected_model_id().is_none());
        assert!(manager.katan_slot_mut()?.can_prompt().is_err());
        Ok(())
    }

    fn manager_with_confirmed_model() -> Result<ProviderManager, Box<dyn std::error::Error>> {
        let mut executor = KatanAgentOllamaExecutor::test_with_models(vec![katan_model()])?;
        executor.select_model_id("confirmed")?;
        let (slot, _events) = KatanSlot::test_ready_slot(executor);
        Ok(manager_with_slot(slot))
    }

    fn streaming_slot(cancelled: bool) -> Result<KatanSlot, Box<dyn std::error::Error>> {
        let mut executor = KatanAgentOllamaExecutor::test_with_models(vec![katan_model()])?;
        executor.select_model_id("confirmed")?;
        let (slot, _events) = KatanSlot::test_streaming_slot(executor, cancelled);
        Ok(slot)
    }

    fn manager_with_slot(slot: KatanSlot) -> ProviderManager {
        let mut manager = ProviderManager::new();
        manager
            .slots
            .insert(KATANAGENT_ID.into(), ProviderSlot::Katan(slot));
        manager
            .inventory
            .ready(KATANAGENT_ID, vec![katan_acp_model()]);
        manager
    }

    fn host(root: &Path) -> Result<ChatHostState, Box<dyn std::error::Error>> {
        Ok(ChatHostState::new(
            ChatSession::new(),
            ChatHistoryStore::new(root.into()),
            ChatOutputExecutor::new(root.into()),
        )?)
    }

    fn assert_selected(host: &ChatHostState, expected: Option<&str>) {
        assert_eq!(
            host.session().vendor_ui_state().selected_model.as_deref(),
            expected
        );
    }

    fn model() -> AcpProviderModel {
        AcpProviderModel {
            config_id: "model".into(),
            id: "provider-default".into(),
            name: "Provider default".into(),
            selected: true,
        }
    }

    fn katan_acp_model() -> AcpProviderModel {
        AcpProviderModel {
            config_id: "ollama-model".into(),
            id: "confirmed".into(),
            name: "Confirmed".into(),
            selected: false,
        }
    }

    fn katan_model() -> KatanAgentOllamaModel {
        KatanAgentOllamaModel {
            id: "confirmed".into(),
            name: "Confirmed".into(),
        }
    }

    fn other_model() -> AcpProviderModel {
        AcpProviderModel {
            config_id: "ollama-model".into(),
            id: "other".into(),
            name: "Other".into(),
            selected: false,
        }
    }
}
