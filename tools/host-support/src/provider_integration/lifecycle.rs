use crate::{AcpProviderModel, ChatHostError, ChatHostState};
use katana_chat_ui::{VendorUiOption, VendorUiState};

use super::permission;

pub(crate) fn ready(
    host: &mut ChatHostState,
    vendor_id: &str,
    models: &[AcpProviderModel],
    preserve_selected_model: bool,
) {
    let mut vendor = vendor_state(host, vendor_id);
    apply_models(&mut vendor, models, preserve_selected_model);
    host.session.set_vendor_ui_state(vendor);
    host.session.set_provider_configured(vendor_id);
}

pub(super) fn connecting(host: &mut ChatHostState) {
    permission::clear(host);
}

pub(super) fn complete(
    host: &mut ChatHostState,
    streaming: &mut bool,
) -> Result<(), ChatHostError> {
    let outcome = if *streaming {
        host.session.finish_assistant_message()
    } else {
        Ok(())
    };
    *streaming = false;
    permission::clear(host);
    outcome.map_err(ChatHostError::from)
}

pub(super) fn fail(
    host: &mut ChatHostState,
    streaming: &mut bool,
    reason: String,
) -> Result<(), ChatHostError> {
    let outcome = if *streaming {
        host.session.fail_assistant_message(reason)
    } else {
        Ok(())
    };
    *streaming = false;
    permission::clear(host);
    host.session
        .set_provider_missing("ACP provider is unavailable");
    outcome.map_err(ChatHostError::from)
}

pub(super) fn config_rejected(
    host: &mut ChatHostState,
    message: String,
) -> Result<(), ChatHostError> {
    // 同じpollで先に確定した設定を、拒否の報告で保存し損ねないようにする。
    host.persist()?;
    Err(ChatHostError::Provider(format!(
        "ACP session configuration rejected: {message}"
    )))
}

fn vendor_state(host: &ChatHostState, vendor_id: &str) -> VendorUiState {
    if host.session.vendor_ui_state().active_vendor_id == vendor_id {
        return host.session.vendor_ui_state().clone();
    }
    VendorUiState::for_vendor(vendor_id)
}

fn apply_models(
    vendor: &mut VendorUiState,
    models: &[AcpProviderModel],
    preserve_selected_model: bool,
) {
    vendor.model_options = models.iter().map(|model| model.id.clone()).collect();
    vendor.model_choices = models
        .iter()
        .map(|model| VendorUiOption {
            id: model.id.clone(),
            label: model.name.clone(),
        })
        .collect();
    if !preserve_selected_model {
        vendor.selected_model = models
            .iter()
            .find(|model| model.selected)
            .map(|model| model.id.clone());
    }
}

#[cfg(test)]
mod tests;
