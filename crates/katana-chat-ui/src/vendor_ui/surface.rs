use serde::{Deserialize, Serialize};

use super::{
    VendorControlProvider, VendorControlRenderModel, VendorFactRegistry, VendorOption,
    VendorUiCapabilities, VendorUiProfile, VendorUiState,
};

type ProfileItems = (
    Option<super::VendorControlItem>,
    Option<super::VendorControlItem>,
    Option<super::VendorControlItem>,
    Option<super::VendorControlItem>,
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorUiSurface {
    #[serde(default)]
    pub pending_permission: Option<super::state::VendorUiPermissionRequest>,
    pub active_vendor_id: String,
    pub active_vendor_label: String,
    pub vendor_options: Vec<VendorOption>,
    pub controls: VendorControlRenderModel,
    pub model_selector_visible: bool,
    pub mode_selector_visible: bool,
    pub thinking_selector_visible: bool,
    pub permission_mode_selector_visible: bool,
    pub tool_approval_visible: bool,
    pub web_search_visible: bool,
    #[serde(default)]
    pub model_choices: Vec<super::VendorUiOption>,
    pub model_labels: Vec<String>,
    pub mode_labels: Vec<String>,
    pub thinking_labels: Vec<String>,
    pub permission_mode_labels: Vec<String>,
}

impl VendorUiSurface {
    pub fn from_registry(registry: &VendorFactRegistry, state: &VendorUiState) -> Self {
        let mut surface = Self::from_controls(registry.vendor_controls(state));
        surface.pending_permission = state.pending_permission.clone();
        surface.set_model_choices(&state.model_choices);
        surface
    }

    pub fn from_capabilities(capabilities: &VendorUiCapabilities) -> Self {
        Self::from_profile(&VendorUiProfile::new(
            "default",
            "Default",
            capabilities.clone(),
        ))
    }

    pub fn from_profile(profile: &VendorUiProfile) -> Self {
        let state = VendorUiState::from_profile(profile);
        let mut surface = Self::from_controls(profile_controls(profile, &state));
        surface.set_model_choices(&state.model_choices);
        surface
    }

    fn from_controls(controls: VendorControlRenderModel) -> Self {
        Self {
            pending_permission: None,
            active_vendor_id: controls.active_vendor_id.clone(),
            active_vendor_label: controls.active_vendor_label.clone(),
            vendor_options: controls.vendor_options.clone(),
            model_selector_visible: controls.model.is_some(),
            mode_selector_visible: controls.mode.is_some(),
            thinking_selector_visible: controls.thinking.is_some(),
            permission_mode_selector_visible: controls.permission.is_some(),
            tool_approval_visible: controls.tools_visible,
            web_search_visible: controls.web_search_visible,
            model_choices: Vec::new(),
            model_labels: labels(&controls.model),
            mode_labels: labels(&controls.mode),
            thinking_labels: labels(&controls.thinking),
            permission_mode_labels: labels(&controls.permission),
            controls,
        }
    }

    fn set_model_choices(&mut self, choices: &[super::VendorUiOption]) {
        self.model_choices = choices.to_vec();
        self.model_labels = model_labels(&self.controls.model, choices);
    }
}

fn profile_controls(profile: &VendorUiProfile, state: &VendorUiState) -> VendorControlRenderModel {
    let (model, mode, thinking, permission) = profile_items(state);
    VendorControlRenderModel {
        active_vendor_id: profile.vendor_id.clone(),
        active_vendor_label: profile.display_name.clone(),
        vendor_options: vec![VendorOption::new(
            profile.vendor_id.clone(),
            profile.display_name.clone(),
        )],
        endpoint: None,
        model,
        mode,
        thinking,
        permission,
        tools_visible: profile.capabilities.tool_approval,
        web_search_visible: profile.capabilities.web_search,
        usage_visible: false,
        account_usage_visible: false,
    }
}

fn profile_items(state: &VendorUiState) -> ProfileItems {
    (
        item(
            "model",
            state.selected_model.clone(),
            state.model_options.clone(),
        ),
        item(
            "mode",
            state.selected_mode.clone(),
            state.mode_options.clone(),
        ),
        item(
            "thinking",
            state.selected_thinking.clone(),
            state.thinking_options.clone(),
        ),
        item(
            "permission",
            state.selected_permission.clone(),
            state.permission_options.clone(),
        ),
    )
}

fn item(
    key: &str,
    selected_value: Option<String>,
    options: Vec<String>,
) -> Option<super::VendorControlItem> {
    if options.is_empty() {
        return None;
    }
    Some(super::VendorControlItem {
        key: key.to_string(),
        label: key.to_string(),
        selected_value,
        options,
        enabled: true,
        disabled_reason: None,
        source_urls: Vec::new(),
    })
}

fn labels(item: &Option<super::VendorControlItem>) -> Vec<String> {
    match item {
        Some(item) => item.options.clone(),
        None => Vec::new(),
    }
}

fn model_labels(
    item: &Option<super::VendorControlItem>,
    choices: &[super::VendorUiOption],
) -> Vec<String> {
    labels(item)
        .iter()
        .map(|value| {
            choices
                .iter()
                .find(|choice| choice.id == *value)
                .map_or_else(|| value.clone(), |choice| choice.label.clone())
        })
        .collect()
}
