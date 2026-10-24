use std::sync::{Arc, Weak};

use kcu_host_support::{ChatHostEffect, ManagedHostUpdate};

use super::MAX_UPDATES_PER_TICK;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct UpdatePlan {
    pub(super) draft: Option<String>,
    pub(super) error: Option<String>,
    pub(super) effect: Option<ChatHostEffect>,
}

pub(super) fn update_plan(latest_draft_seq: u64, update: &ManagedHostUpdate) -> UpdatePlan {
    UpdatePlan {
        draft: (update.ack_seq >= latest_draft_seq).then(|| update.surface.composer.text.clone()),
        error: update.error.clone(),
        effect: update.effect.clone(),
    }
}

pub(super) fn drain_bounded<T, E>(
    mut next: impl FnMut() -> Result<Option<T>, E>,
    mut apply: impl FnMut(T),
    mut failed: impl FnMut(E),
) {
    for _ in 0..MAX_UPDATES_PER_TICK {
        match next() {
            Ok(Some(value)) => apply(value),
            Ok(None) => return,
            Err(error) => {
                failed(error);
                return;
            }
        }
    }
}

pub(super) fn upgrade_host<T>(host: &Weak<T>) -> Option<Arc<T>> {
    host.upgrade()
}
