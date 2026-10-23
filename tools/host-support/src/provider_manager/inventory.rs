use std::collections::BTreeMap;

use crate::AcpProviderModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ProviderSlotState {
    Connecting,
    Ready(Vec<AcpProviderModel>),
    Unavailable(String),
    Stopped(String),
}

#[derive(Default)]
pub(super) struct ProviderInventory {
    slots: BTreeMap<String, ProviderSlotState>,
}

impl ProviderInventory {
    pub(super) fn connecting(&mut self, vendor_id: &str) {
        self.slots
            .insert(vendor_id.to_string(), ProviderSlotState::Connecting);
    }

    pub(super) fn ready(&mut self, vendor_id: &str, models: Vec<AcpProviderModel>) {
        self.slots
            .insert(vendor_id.to_string(), ProviderSlotState::Ready(models));
    }

    pub(super) fn unavailable(&mut self, vendor_id: &str, reason: impl Into<String>) {
        self.slots.insert(
            vendor_id.to_string(),
            ProviderSlotState::Unavailable(reason.into()),
        );
    }

    pub(super) fn stopped(&mut self, vendor_id: &str, reason: impl Into<String>) {
        self.slots.insert(
            vendor_id.to_string(),
            ProviderSlotState::Stopped(reason.into()),
        );
    }

    pub(super) fn ready_models(&self, vendor_id: &str) -> Option<Vec<AcpProviderModel>> {
        match self.slots.get(vendor_id) {
            Some(ProviderSlotState::Ready(models)) => Some(models.clone()),
            _ => None,
        }
    }

    pub(super) fn available_vendor_ids(&self) -> Vec<String> {
        self.slots
            .iter()
            .filter(|(_, state)| matches!(state, ProviderSlotState::Ready(_)))
            .map(|(vendor_id, _)| vendor_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ProviderInventory;
    use crate::AcpProviderModel;

    #[test]
    fn only_ready_slots_are_available_for_selection() {
        let mut inventory = ProviderInventory::default();
        inventory.connecting("codex-cli");
        inventory.unavailable("claude-code", "CLI is missing");
        inventory.ready("github-copilot", vec![model()]);

        assert_eq!(
            inventory.available_vendor_ids(),
            vec!["github-copilot".to_string()]
        );
        assert!(inventory.ready_models("codex-cli").is_none());
        assert_eq!(
            inventory.ready_models("github-copilot"),
            Some(vec![model()])
        );
    }

    fn model() -> AcpProviderModel {
        AcpProviderModel {
            config_id: "model".into(),
            id: "copilot-model".into(),
            name: "Copilot model".into(),
            selected: true,
        }
    }
}
