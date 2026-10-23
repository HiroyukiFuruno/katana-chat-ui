use katana_chat_ui::{VendorUiOption, VendorUiState};

use crate::{AcpProviderModel, ChatHostError, ChatHostState};

use super::{KATANAGENT_ID, ProviderManager, katan_request};

impl ProviderManager {
    pub(super) fn select_katan(
        &mut self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        self.configure_katan(host, models, None)
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
    use super::{KATANAGENT_ID, ProviderManager};
    use crate::{AcpProviderModel, ChatHistoryStore, ChatHostState, ChatOutputExecutor};
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

    fn model() -> AcpProviderModel {
        AcpProviderModel {
            config_id: "model".into(),
            id: "provider-default".into(),
            name: "Provider default".into(),
            selected: true,
        }
    }
}
