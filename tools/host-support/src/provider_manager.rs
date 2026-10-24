use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    AcpHostBridge, AcpProviderExecutor, AcpProviderProjection, ChatHostError, ChatHostState,
    acp_provider_profile,
};

mod commands;
mod inventory;
mod katan_commands;
mod katan_poll;
mod katan_request;
mod katan_worker;
mod katanagent;
mod poll;
mod routing;
mod session_lifecycle;

#[cfg(test)]
mod tests;

use inventory::ProviderInventory;
use katanagent::{KatanEvent, KatanSlot};

const ACP_VENDOR_IDS: [&str; 3] = ["codex-cli", "claude-code", "github-copilot"];
const KATANAGENT_ID: &str = "katanagent";
const MAX_PROJECTIONS_PER_POLL: usize = 32;

#[derive(Default)]
pub struct ProviderManager {
    workspace_root: PathBuf,
    slots: BTreeMap<String, ProviderSlot>,
    inventory: ProviderInventory,
    active_vendor_id: Option<String>,
    restored_model: Option<RestoredModelSelection>,
    pending_cancellations: Vec<PendingCancellation>,
}

struct AcpSlot {
    bridge: AcpHostBridge,
}

enum ProviderSlot {
    Acp(Box<AcpSlot>),
    Katan(KatanSlot),
}

enum ProviderSlotEvent {
    Acp(AcpProviderProjection),
    Katan(KatanEvent),
}

struct PendingCancellation {
    vendor_id: String,
    request_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RestoredModelSelection {
    Waiting {
        vendor_id: String,
        model_id: String,
    },
    Applying {
        vendor_id: String,
        model_id: String,
        config_id: String,
    },
    AwaitingConfirmation {
        vendor_id: String,
        model_id: String,
        config_id: String,
    },
    Blocked {
        vendor_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderPollEvent {
    pub vendor_id: String,
    pub kind: ProviderPollKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderPollKind {
    Ready,
    AppliedToActive,
    IgnoredFromInactive,
    InactivePermissionPending,
    CancelledInactivePermission,
    Unavailable,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            workspace_root: PathBuf::new(),
            slots: BTreeMap::new(),
            inventory: ProviderInventory::default(),
            active_vendor_id: None,
            restored_model: None,
            pending_cancellations: Vec::new(),
        }
    }

    pub fn probe(&mut self, root: impl Into<PathBuf>) -> Vec<ProviderPollEvent> {
        let root = root.into();
        self.workspace_root = root.clone();
        let mut events = Vec::new();
        for vendor_id in ACP_VENDOR_IDS {
            self.connect(vendor_id, &root, &mut events);
        }
        self.connect_katanagent();
        events
    }

    pub async fn poll(
        &mut self,
        host: &mut ChatHostState,
    ) -> Result<Vec<ProviderPollEvent>, ChatHostError> {
        let mut events = self.try_poll(host)?;
        self.flush_restored_model_application(host).await?;
        events.extend(self.flush_pending_cancellations().await?);
        Ok(events)
    }

    pub(crate) fn try_poll(
        &mut self,
        host: &mut ChatHostState,
    ) -> Result<Vec<ProviderPollEvent>, ChatHostError> {
        let vendor_ids = self.slots.keys().cloned().collect::<Vec<_>>();
        let mut events = Vec::new();
        let mut processed = 0;
        let mut idle_slots = 0;
        let mut slot_index = 0;
        while processed < MAX_PROJECTIONS_PER_POLL && idle_slots < vendor_ids.len() {
            let vendor_id = &vendor_ids[slot_index % vendor_ids.len()];
            slot_index += 1;
            if let Some(slot_event) = self.next_slot_event(vendor_id) {
                self.handle_slot_event(host, vendor_id, slot_event, &mut events)?;
                processed += 1;
                idle_slots = 0;
            } else {
                idle_slots += 1;
            }
        }
        if !events.is_empty() {
            host.persist()?;
        }
        Ok(events)
    }

    pub fn available_vendor_ids(&self) -> Vec<String> {
        self.inventory.available_vendor_ids()
    }

    fn connect(&mut self, vendor_id: &str, root: &Path, events: &mut Vec<ProviderPollEvent>) {
        let result = acp_provider_profile(vendor_id, root.to_path_buf())
            .and_then(|profile| AcpProviderExecutor::connect(profile).map_err(provider_error));
        match result {
            Ok(executor) => {
                self.slots.insert(
                    vendor_id.to_string(),
                    ProviderSlot::Acp(Box::new(AcpSlot {
                        bridge: AcpHostBridge::new(executor),
                    })),
                );
                self.inventory.connecting(vendor_id);
            }
            Err(error) => {
                self.inventory.unavailable(vendor_id, error.to_string());
                events.push(event(vendor_id, ProviderPollKind::Unavailable));
            }
        }
    }

    fn connect_katanagent(&mut self) {
        let endpoint = std::env::var("KCU_OLLAMA_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "http://localhost:11434".into());
        self.inventory.connecting(KATANAGENT_ID);
        self.slots.insert(
            KATANAGENT_ID.into(),
            ProviderSlot::Katan(KatanSlot::probe(endpoint)),
        );
    }

    fn next_slot_event(&mut self, vendor_id: &str) -> Option<ProviderSlotEvent> {
        match self.slots.get_mut(vendor_id)? {
            ProviderSlot::Acp(slot) => slot
                .bridge
                .try_next_projection()
                .map(ProviderSlotEvent::Acp),
            ProviderSlot::Katan(slot) => slot.try_next_event().map(ProviderSlotEvent::Katan),
        }
    }
}

fn event(vendor_id: &str, kind: ProviderPollKind) -> ProviderPollEvent {
    ProviderPollEvent {
        vendor_id: vendor_id.to_string(),
        kind,
    }
}

fn provider_error(error: katana_acp_client::AcpError) -> ChatHostError {
    ChatHostError::Provider(error.to_string())
}
