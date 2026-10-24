use crate::{AcpProviderProjection, ChatHostError, ChatHostState};

use super::{
    PendingCancellation, ProviderManager, ProviderPollEvent, ProviderPollKind, ProviderSlot,
    ProviderSlotEvent, event,
    routing::{ProjectionRoute, route},
};

impl ProviderManager {
    pub(super) fn handle_slot_event(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        event: ProviderSlotEvent,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        match event {
            ProviderSlotEvent::Acp(projection) => {
                self.handle_projection(host, vendor_id, projection, events)
            }
            ProviderSlotEvent::Katan(event) => self.handle_katan_event(host, event, events),
        }
    }

    pub(super) fn handle_projection(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        projection: AcpProviderProjection,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        if let AcpProviderProjection::ConfigUpdated { config_id } = &projection {
            return self.confirm_restored_model(host, vendor_id, config_id, events);
        }
        if let AcpProviderProjection::ConfigRejected { config_id, message } = &projection
            && self
                .restored_model_confirmation(vendor_id)
                .is_some_and(|(expected_config, _)| expected_config == *config_id)
        {
            return self.reject_restored_model(host, vendor_id, message.clone());
        }
        match route(self.active_vendor_id.as_deref(), vendor_id, &projection) {
            ProjectionRoute::Ready => self.record_ready(host, vendor_id, projection, events),
            ProjectionRoute::ApplyActive => self.apply_active(host, vendor_id, projection, events),
            ProjectionRoute::IgnoreInactive => {
                self.ignore_inactive(host, vendor_id, projection, events)
            }
            ProjectionRoute::CancelInactivePermission => {
                self.stop_inactive_permission(host, vendor_id, projection, events)
            }
        }
    }

    fn confirm_restored_model(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        config_id: &str,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        if self.active_vendor_id.as_deref() != Some(vendor_id) {
            events.push(event(vendor_id, ProviderPollKind::IgnoredFromInactive));
            return Ok(());
        }
        let Some((expected_config, expected_model)) = self.restored_model_confirmation(vendor_id)
        else {
            events.push(event(vendor_id, ProviderPollKind::AppliedToActive));
            return Ok(());
        };
        if config_id != expected_config {
            events.push(event(vendor_id, ProviderPollKind::AppliedToActive));
            return Ok(());
        }

        self.complete_restored_model_confirmation(
            host,
            vendor_id,
            &expected_config,
            &expected_model,
            events,
        )
    }

    fn complete_restored_model_confirmation(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        expected_config: &str,
        expected_model: &str,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        let models = self.inventory.ready_models(vendor_id).unwrap_or_default();
        let selected_model = models
            .iter()
            .find(|model| model.config_id == expected_config && model.selected);
        if selected_model.is_some_and(|model| model.id == expected_model) {
            self.clear_restored_model();
            crate::provider_integration::lifecycle::ready(host, vendor_id, &models, false);
            events.push(event(vendor_id, ProviderPollKind::AppliedToActive));
            return Ok(());
        }

        let reason = match selected_model {
            Some(model) => format!(
                "restored model confirmation mismatch: requested {expected_model}, agent selected {}",
                model.id
            ),
            None => format!(
                "restored model confirmation missing: agent did not select {expected_model}"
            ),
        };
        self.block_restored_model(reason.clone());
        host.session_mut().set_provider_missing(reason.clone());
        host.persist()?;
        Err(ChatHostError::Provider(reason))
    }

    fn reject_restored_model(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        message: String,
    ) -> Result<(), ChatHostError> {
        let (_, model_id) = self
            .restored_model_confirmation(vendor_id)
            .ok_or_else(|| ChatHostError::Provider("restored model state disappeared".into()))?;
        let reason = format!("restored model {model_id} was rejected: {message}");
        self.block_restored_model(reason.clone());
        host.session_mut().set_provider_missing(reason.clone());
        host.persist()?;
        Err(ChatHostError::Provider(reason))
    }

    fn record_ready(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        projection: AcpProviderProjection,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        let AcpProviderProjection::Ready { models } = projection else {
            return Ok(());
        };
        self.inventory.ready(vendor_id, models.clone());
        if self.active_vendor_id.as_deref() == Some(vendor_id) {
            self.apply_ready_to_active(host, vendor_id, models)?;
        }
        self.set_restored_model_ready_state(host, vendor_id);
        self.sync_inventory(host);
        events.push(event(vendor_id, ProviderPollKind::Ready));
        Ok(())
    }

    fn apply_active(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        projection: AcpProviderProjection,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        let unavailable = unavailable_reason(&projection);
        self.apply_to_active(host, vendor_id, projection)?;
        if let Some(reason) = unavailable {
            self.inventory.unavailable(vendor_id, reason);
            self.sync_inventory(host);
            events.push(event(vendor_id, ProviderPollKind::Unavailable));
        } else {
            events.push(event(vendor_id, ProviderPollKind::AppliedToActive));
        }
        Ok(())
    }

    fn ignore_inactive(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        projection: AcpProviderProjection,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        if let Some(reason) = unavailable_reason(&projection) {
            self.inventory.unavailable(vendor_id, reason);
            self.sync_inventory(host);
            events.push(event(vendor_id, ProviderPollKind::Unavailable));
        } else {
            events.push(event(vendor_id, ProviderPollKind::IgnoredFromInactive));
        }
        Ok(())
    }

    fn stop_inactive_permission(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        projection: AcpProviderProjection,
        events: &mut Vec<ProviderPollEvent>,
    ) -> Result<(), ChatHostError> {
        let AcpProviderProjection::Permission { request_id, .. } = projection else {
            return Ok(());
        };
        self.pending_cancellations.push(PendingCancellation {
            vendor_id: vendor_id.to_string(),
            request_id,
        });
        self.inventory
            .stopped(vendor_id, "non-active ACP permission request");
        self.sync_inventory(host);
        events.push(event(
            vendor_id,
            ProviderPollKind::InactivePermissionPending,
        ));
        Ok(())
    }

    pub(super) fn apply_to_active(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        projection: AcpProviderProjection,
    ) -> Result<(), ChatHostError> {
        let slot = self
            .slots
            .get_mut(vendor_id)
            .ok_or_else(|| ChatHostError::Provider(format!("ACP slot disappeared: {vendor_id}")))?;
        let ProviderSlot::Acp(slot) = slot else {
            return Err(ChatHostError::Provider(format!(
                "ACP projection sent to direct provider: {vendor_id}"
            )));
        };
        slot.bridge.apply_projection(host, projection)
    }

    fn apply_ready_to_active(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
        models: Vec<crate::AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        let preserve_restored_model = self.should_preserve_restored_model(vendor_id);
        let slot = self
            .slots
            .get_mut(vendor_id)
            .ok_or_else(|| ChatHostError::Provider(format!("ACP slot disappeared: {vendor_id}")))?;
        let ProviderSlot::Acp(slot) = slot else {
            return Err(ChatHostError::Provider(format!(
                "ACP projection sent to direct provider: {vendor_id}"
            )));
        };
        if preserve_restored_model {
            slot.bridge.apply_restored_ready_projection(host, models)
        } else {
            slot.bridge
                .apply_projection(host, AcpProviderProjection::Ready { models })
        }
    }

    pub(super) fn sync_inventory(&self, host: &mut ChatHostState) {
        let mut vendor = host.session.vendor_ui_state().clone();
        vendor.available_vendor_ids = self.inventory.available_vendor_ids();
        host.session.set_vendor_ui_state(vendor);
    }

    pub(super) fn set_restored_model_ready_state(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
    ) {
        let Some(models) = self.inventory.ready_models(vendor_id) else {
            return;
        };
        match self.restored_model_ready(vendor_id, &models) {
            Some(Ok(_)) => host
                .session_mut()
                .set_provider_missing("restored model is being applied"),
            Some(Err(reason)) => host.session_mut().set_provider_missing(reason),
            None => {}
        }
    }

    pub(super) async fn flush_restored_model_application(
        &mut self,
        host: &mut ChatHostState,
    ) -> Result<(), ChatHostError> {
        let Some((vendor_id, model_id)) = self.restored_model_application() else {
            return Ok(());
        };
        if self.active_vendor_id.as_deref() != Some(vendor_id.as_str()) {
            return Ok(());
        }
        match self.select_model(host, &model_id).await {
            Ok(()) => {}
            Err(error) => {
                let reason = format!("restored model could not be applied: {model_id}: {error}");
                self.block_restored_model(reason.clone());
                host.session_mut().set_provider_missing(reason);
            }
        }
        host.persist()
    }
}

fn unavailable_reason(projection: &AcpProviderProjection) -> Option<String> {
    match projection {
        AcpProviderProjection::Unavailable(reason) | AcpProviderProjection::Failed(reason) => {
            Some(reason.clone())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
