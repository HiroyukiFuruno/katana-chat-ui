use super::{KATANAGENT_ID, KatanEvent, KatanSlot, ProviderManager, ProviderSlot};

#[test]
fn discarding_a_manager_slot_rejects_its_queued_and_late_events() -> Result<(), String> {
    let (slot, sender) = KatanSlot::test_slot();
    let mut manager = ProviderManager::new();
    manager
        .slots
        .insert(KATANAGENT_ID.into(), ProviderSlot::Katan(slot));
    sender
        .send(KatanEvent::CatalogFailed("old session".into()))
        .map_err(|error| error.to_string())?;
    manager.discard_slots();
    if manager.next_slot_event(KATANAGENT_ID).is_some() {
        return Err("discarded manager slot still yielded an event".into());
    }
    if sender
        .send(KatanEvent::CatalogFailed("late session".into()))
        .is_ok()
    {
        return Err("discarded manager slot accepted a late event".into());
    }
    Ok(())
}
