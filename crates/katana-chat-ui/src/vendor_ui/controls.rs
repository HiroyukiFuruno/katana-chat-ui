use serde::{Deserialize, Serialize};

use super::{
    VendorCapabilityFact, VendorCapabilityStatus, VendorFact, VendorFactRegistry, VendorOption,
    VendorUiState,
};

pub trait VendorControlProvider {
    fn vendor_controls(&self, state: &VendorUiState) -> VendorControlRenderModel;
}

impl VendorControlProvider for VendorFactRegistry {
    fn vendor_controls(&self, state: &VendorUiState) -> VendorControlRenderModel {
        VendorControlRenderModel::from_registry(self, state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorControlRenderModel {
    pub active_vendor_id: String,
    pub active_vendor_label: String,
    pub vendor_options: Vec<VendorOption>,
    pub endpoint: Option<VendorControlItem>,
    pub model: Option<VendorControlItem>,
    pub mode: Option<VendorControlItem>,
    pub thinking: Option<VendorControlItem>,
    pub permission: Option<VendorControlItem>,
    pub tools_visible: bool,
    pub web_search_visible: bool,
    pub usage_visible: bool,
    pub account_usage_visible: bool,
}

impl VendorControlRenderModel {
    pub fn from_registry(registry: &VendorFactRegistry, state: &VendorUiState) -> Self {
        let fact = registry.get(&state.active_vendor_id);
        match fact {
            Some(fact) => Self::from_fact(fact, state, registry),
            None => Self::unknown(state),
        }
    }

    fn from_fact(fact: &VendorFact, state: &VendorUiState, registry: &VendorFactRegistry) -> Self {
        Self {
            active_vendor_id: fact.vendor_id.clone(),
            active_vendor_label: fact.display_name.clone(),
            vendor_options: vendor_options(state, registry),
            endpoint: control(&fact.endpoint, "endpoint", &state.endpoint, &[]),
            model: Self::model_control(fact, state),
            mode: Self::mode_control(fact, state),
            thinking: Self::thinking_control(fact, state),
            permission: Self::permission_control(fact, state),
            tools_visible: fact.tools.supported_by_official_reference(),
            web_search_visible: fact.web_search.supported_by_official_reference(),
            usage_visible: fact.usage.supported_by_official_reference(),
            account_usage_visible: fact.account_usage.supported_by_official_reference(),
        }
    }

    fn model_control(fact: &VendorFact, state: &VendorUiState) -> Option<VendorControlItem> {
        control(
            &fact.model,
            "model",
            &state.selected_model,
            &state.model_options,
        )
    }

    fn mode_control(fact: &VendorFact, state: &VendorUiState) -> Option<VendorControlItem> {
        control(
            &fact.mode,
            "mode",
            &state.selected_mode,
            &state.mode_options,
        )
    }

    fn thinking_control(fact: &VendorFact, state: &VendorUiState) -> Option<VendorControlItem> {
        control(
            &fact.thinking,
            "thinking",
            &state.selected_thinking,
            &state.thinking_options,
        )
    }

    fn permission_control(fact: &VendorFact, state: &VendorUiState) -> Option<VendorControlItem> {
        control(
            &fact.permission,
            "permission",
            &state.selected_permission,
            &state.permission_options,
        )
    }

    fn unknown(state: &VendorUiState) -> Self {
        Self {
            active_vendor_id: state.active_vendor_id.clone(),
            active_vendor_label: state.active_vendor_id.clone(),
            vendor_options: Vec::new(),
            endpoint: None,
            model: None,
            mode: None,
            thinking: None,
            permission: None,
            tools_visible: false,
            web_search_visible: false,
            usage_visible: false,
            account_usage_visible: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorControlItem {
    pub key: String,
    pub label: String,
    pub selected_value: Option<String>,
    pub options: Vec<String>,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub source_urls: Vec<String>,
}

fn vendor_options(state: &VendorUiState, registry: &VendorFactRegistry) -> Vec<VendorOption> {
    state
        .available_vendor_ids
        .iter()
        .filter_map(|vendor_id| registry.get(vendor_id))
        .map(|it| VendorOption::new(it.vendor_id.clone(), it.display_name.clone()))
        .collect::<Vec<_>>()
}

fn control(
    fact: &VendorCapabilityFact,
    key: &str,
    selected: &Option<String>,
    runtime_options: &[String],
) -> Option<VendorControlItem> {
    if fact.status != VendorCapabilityStatus::Supported {
        return None;
    }
    if fact.references.is_empty() {
        return None;
    }
    let options = options(fact, runtime_options);
    let enabled = selected.is_some() || !options.is_empty();
    Some(VendorControlItem {
        key: key.to_string(),
        label: key.to_string(),
        selected_value: selected.clone(),
        options,
        enabled,
        disabled_reason: disabled_reason(enabled),
        source_urls: fact.references.iter().map(|it| it.url.clone()).collect(),
    })
}

fn options(fact: &VendorCapabilityFact, runtime_options: &[String]) -> Vec<String> {
    if runtime_options.is_empty() {
        return fact.options.clone();
    }
    runtime_options.to_vec()
}

fn disabled_reason(enabled: bool) -> Option<String> {
    if enabled {
        return None;
    }
    Some("設定値が未取得です".to_string())
}
