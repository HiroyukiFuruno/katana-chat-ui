use kcu_host_support::ChatHostEffect;

pub(super) fn handle_effect(effect: ChatHostEffect) -> Result<(), String> {
    match effect {
        ChatHostEffect::None
        | ChatHostEffect::OutputApplied { .. }
        | ChatHostEffect::OutputReverted { .. } => Ok(()),
        ChatHostEffect::RequestAttachment => Err("Attachment picker is not available".into()),
        ChatHostEffect::ProviderSelectionRequired { vendor_id } => {
            Err(format!("Provider connection is not available: {vendor_id}"))
        }
        ChatHostEffect::Submitted { .. } => Err("No connected provider received the prompt".into()),
    }
}
