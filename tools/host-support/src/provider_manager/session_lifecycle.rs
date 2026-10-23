use crate::{AcpPermissionDecision, AcpProviderModel, ChatHostError, ChatHostState};

use super::{ProviderInventory, ProviderManager, RestoredModelSelection};

impl ProviderManager {
    /// 現在の会話に属する provider transport と未処理 event を破棄する。
    pub async fn begin_session_boundary(
        &mut self,
        host: &mut ChatHostState,
    ) -> Result<(), ChatHostError> {
        self.cancel_active_work(host).await?;
        self.discard_slots();
        self.probe(self.workspace_root.clone());
        self.sync_inventory(host);
        host.persist()
    }

    pub fn bind_session_identity(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        let vendor_id = host.session().vendor_ui_state().active_vendor_id.clone();
        self.active_vendor_id = self
            .slots
            .contains_key(&vendor_id)
            .then_some(vendor_id.clone());
        self.history_context_pending = !host.session().snapshot().messages.is_empty();
        self.restore_model_selection(host, &vendor_id);
        self.install_history_context(host);
        self.set_restored_connection_state(host, &vendor_id);
        self.sync_inventory(host);
        host.persist()
    }

    async fn cancel_active_work(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        if let Some(request) = host.session().vendor_ui_state().pending_permission.clone() {
            self.respond_permission(host, request.request_id, AcpPermissionDecision::Cancelled)
                .await?;
        }
        if host.session().render_model().input.can_cancel {
            self.stop(host).await?;
        }
        Ok(())
    }

    pub(super) fn discard_slots(&mut self) {
        self.slots.clear();
        self.inventory = ProviderInventory::default();
        self.history_context_pending = false;
        self.restored_model = None;
        self.pending_cancellations.clear();
    }

    pub(super) fn install_history_context(&mut self, host: &ChatHostState) {
        if !self.history_context_pending {
            return;
        }
        let Some(vendor_id) = self.active_vendor_id.clone() else {
            return;
        };
        let Some(super::ProviderSlot::Acp(slot)) = self.slots.get_mut(&vendor_id) else {
            return;
        };
        slot.bridge.set_history_context(host);
    }

    pub(super) fn consume_history_context(&mut self) {
        self.history_context_pending = false;
        for slot in self.slots.values_mut() {
            if let super::ProviderSlot::Acp(slot) = slot {
                slot.bridge.clear_history_context();
            }
        }
    }

    fn set_restored_connection_state(&self, host: &mut ChatHostState, vendor_id: &str) {
        let reason = if self.active_vendor_id.is_some() {
            "provider is reconnecting".into()
        } else if vendor_id.is_empty() {
            "provider is not configured".into()
        } else {
            format!("restored provider is unavailable: {vendor_id}")
        };
        host.session_mut().set_provider_missing(reason);
    }

    fn restore_model_selection(&mut self, host: &ChatHostState, vendor_id: &str) {
        self.restored_model =
            host.session()
                .vendor_ui_state()
                .selected_model
                .clone()
                .map(|model_id| RestoredModelSelection::Waiting {
                    vendor_id: vendor_id.into(),
                    model_id,
                });
    }

    pub(super) fn restored_model_ready(
        &mut self,
        vendor_id: &str,
        models: &[AcpProviderModel],
    ) -> Option<Result<String, String>> {
        if let Some(reason) = self.restored_model_block_reason(vendor_id) {
            return Some(Err(reason));
        }
        if self.restored_model_is_applying(vendor_id)
            || self.restored_model_is_awaiting_confirmation(vendor_id)
        {
            return Some(Err("restored model is awaiting agent confirmation".into()));
        }
        let model_id = self.restored_model_waiting_id(vendor_id)?;
        if let Some(model) = models.iter().find(|model| model.id == model_id) {
            self.restored_model = Some(RestoredModelSelection::Applying {
                vendor_id: vendor_id.into(),
                model_id: model_id.clone(),
                config_id: model.config_id.clone(),
            });
            Some(Ok(model_id))
        } else {
            let reason = format!("restored model is unavailable: {model_id}");
            self.restored_model = Some(RestoredModelSelection::Blocked {
                vendor_id: vendor_id.into(),
                reason: reason.clone(),
            });
            Some(Err(reason))
        }
    }

    fn restored_model_block_reason(&self, vendor_id: &str) -> Option<String> {
        match self.restored_model.as_ref()? {
            RestoredModelSelection::Blocked {
                vendor_id: restored,
                reason,
            } if restored == vendor_id => Some(reason.clone()),
            _ => None,
        }
    }

    fn restored_model_waiting_id(&self, vendor_id: &str) -> Option<String> {
        match self.restored_model.as_ref()? {
            RestoredModelSelection::Waiting {
                vendor_id: restored,
                model_id,
            } if restored == vendor_id => Some(model_id.clone()),
            _ => None,
        }
    }

    fn restored_model_is_applying(&self, vendor_id: &str) -> bool {
        matches!(
            self.restored_model.as_ref(),
            Some(RestoredModelSelection::Applying { vendor_id: restored, .. })
                if restored == vendor_id
        )
    }

    pub(super) fn restored_model_application(&self) -> Option<(String, String)> {
        match self.restored_model.as_ref()? {
            RestoredModelSelection::Applying {
                vendor_id,
                model_id,
                ..
            } => Some((vendor_id.clone(), model_id.clone())),
            _ => None,
        }
    }

    pub(super) fn restored_model_confirmation(&self, vendor_id: &str) -> Option<(String, String)> {
        match self.restored_model.as_ref()? {
            RestoredModelSelection::AwaitingConfirmation {
                vendor_id: restored,
                model_id,
                config_id,
            } if restored == vendor_id => Some((config_id.clone(), model_id.clone())),
            _ => None,
        }
    }

    pub(super) fn await_restored_model_confirmation(
        &mut self,
        vendor_id: &str,
        config_id: String,
        model_id: String,
    ) {
        self.restored_model = Some(RestoredModelSelection::AwaitingConfirmation {
            vendor_id: vendor_id.into(),
            model_id,
            config_id,
        });
    }

    pub(super) fn block_restored_model(&mut self, reason: String) {
        let vendor_id = match self.restored_model.as_ref() {
            Some(RestoredModelSelection::Applying { vendor_id, .. })
            | Some(RestoredModelSelection::AwaitingConfirmation { vendor_id, .. }) => {
                vendor_id.clone()
            }
            _ => return,
        };
        self.restored_model = Some(RestoredModelSelection::Blocked { vendor_id, reason });
    }

    pub(super) fn should_preserve_restored_model(&self, vendor_id: &str) -> bool {
        matches!(
            self.restored_model.as_ref(),
            Some(RestoredModelSelection::Waiting { vendor_id: restored, .. }
                | RestoredModelSelection::Applying { vendor_id: restored, .. }
                | RestoredModelSelection::AwaitingConfirmation { vendor_id: restored, .. }
                | RestoredModelSelection::Blocked { vendor_id: restored, .. })
                if restored == vendor_id
        )
    }

    fn restored_model_is_awaiting_confirmation(&self, vendor_id: &str) -> bool {
        matches!(
            self.restored_model.as_ref(),
            Some(RestoredModelSelection::AwaitingConfirmation { vendor_id: restored, .. })
                if restored == vendor_id
        )
    }

    pub(super) fn clear_restored_model(&mut self) {
        self.restored_model = None;
    }

    pub(super) fn restored_model_is_pending(&self) -> bool {
        self.restored_model.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{ProviderManager, RestoredModelSelection};
    use crate::AcpProviderModel;

    #[test]
    fn consumed_history_is_not_scheduled_again_by_same_provider_selection() -> Result<(), String> {
        let mut manager = ProviderManager::new();
        manager.history_context_pending = true;
        manager.consume_history_context();
        if manager.history_context_pending {
            return Err("consumed history remained pending".into());
        }
        Ok(())
    }

    #[test]
    fn restored_model_waits_for_matching_ready_catalog() {
        let mut manager = ProviderManager::new();
        manager.restored_model = Some(RestoredModelSelection::Waiting {
            vendor_id: "katanagent".into(),
            model_id: "saved".into(),
        });
        assert_eq!(
            manager.restored_model_ready("codex-cli", &[model("saved")]),
            None
        );
        assert_eq!(
            manager.restored_model_ready("katanagent", &[model("saved")]),
            Some(Ok("saved".into()))
        );
        assert_eq!(
            manager.restored_model_application(),
            Some(("katanagent".into(), "saved".into()))
        );
    }

    #[test]
    fn missing_restored_model_does_not_fallback_to_provider_default() {
        let mut manager = ProviderManager::new();
        manager.restored_model = Some(RestoredModelSelection::Waiting {
            vendor_id: "codex-cli".into(),
            model_id: "saved".into(),
        });
        assert_eq!(
            manager.restored_model_ready("codex-cli", &[model("provider-default")]),
            Some(Err("restored model is unavailable: saved".into()))
        );
        assert!(manager.restored_model_is_pending());
        assert_eq!(
            manager.restored_model_ready("codex-cli", &[model("provider-default")]),
            Some(Err("restored model is unavailable: saved".into()))
        );
    }

    #[test]
    fn explicit_selection_and_session_boundary_discard_restored_model() {
        let mut manager = ProviderManager::new();
        manager.restored_model = Some(RestoredModelSelection::Waiting {
            vendor_id: "codex-cli".into(),
            model_id: "saved".into(),
        });
        manager.clear_restored_model();
        assert!(!manager.restored_model_is_pending());
        manager.restored_model = Some(RestoredModelSelection::Waiting {
            vendor_id: "codex-cli".into(),
            model_id: "saved".into(),
        });
        manager.discard_slots();
        assert!(!manager.restored_model_is_pending());
    }

    #[test]
    fn failed_restored_model_application_stays_blocked_across_ready_events() {
        let mut manager = ProviderManager::new();
        manager.restored_model = Some(RestoredModelSelection::Applying {
            vendor_id: "codex-cli".into(),
            model_id: "saved".into(),
            config_id: "model".into(),
        });
        manager.block_restored_model("restored model could not be applied: saved".into());
        assert_eq!(
            manager.restored_model_ready("codex-cli", &[model("provider-default")]),
            Some(Err("restored model could not be applied: saved".into()))
        );
        assert!(manager.restored_model_is_pending());
    }

    fn model(id: &str) -> AcpProviderModel {
        AcpProviderModel {
            config_id: "model".into(),
            id: id.into(),
            name: id.into(),
            selected: true,
        }
    }
}
