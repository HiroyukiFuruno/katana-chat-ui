use serde::{Deserialize, Serialize};

use super::legacy::VendorUiProfile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorUiState {
    pub active_vendor_id: String,
    #[serde(default)]
    pub available_vendor_ids: Vec<String>,
    pub endpoint: Option<String>,
    pub selected_model: Option<String>,
    pub selected_mode: Option<String>,
    pub selected_thinking: Option<String>,
    pub selected_permission: Option<String>,
    pub model_options: Vec<String>,
    pub mode_options: Vec<String>,
    pub thinking_options: Vec<String>,
    pub permission_options: Vec<String>,
}

impl VendorUiState {
    pub fn for_vendor(vendor_id: impl Into<String>) -> Self {
        let active_vendor_id = vendor_id.into();
        Self {
            active_vendor_id: active_vendor_id.clone(),
            available_vendor_ids: vec![active_vendor_id],
            endpoint: None,
            selected_model: None,
            selected_mode: None,
            selected_thinking: None,
            selected_permission: None,
            model_options: Vec::new(),
            mode_options: Vec::new(),
            thinking_options: Vec::new(),
            permission_options: Vec::new(),
        }
    }

    pub fn without_vendor() -> Self {
        Self {
            active_vendor_id: String::new(),
            available_vendor_ids: Vec::new(),
            endpoint: None,
            selected_model: None,
            selected_mode: None,
            selected_thinking: None,
            selected_permission: None,
            model_options: Vec::new(),
            mode_options: Vec::new(),
            thinking_options: Vec::new(),
            permission_options: Vec::new(),
        }
    }

    pub fn from_profile(profile: &VendorUiProfile) -> Self {
        let capabilities = &profile.capabilities;
        let mut state = Self::for_vendor(profile.vendor_id.clone());
        state.model_options = capabilities.model_labels.clone();
        state.mode_options = capabilities.mode_labels.clone();
        state.thinking_options = capabilities.thinking_labels.clone();
        state.permission_options = capabilities.permission_mode_labels.clone();
        state.selected_model = state.model_options.first().cloned();
        state.selected_mode = state.mode_options.first().cloned();
        state.selected_thinking = state.thinking_options.first().cloned();
        state.selected_permission = state.permission_options.first().cloned();
        state
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn with_models(mut self, options: Vec<String>, selected: impl Into<String>) -> Self {
        self.model_options = options;
        self.selected_model = Some(selected.into());
        self
    }

    pub fn with_thinking(mut self, options: Vec<String>, selected: impl Into<String>) -> Self {
        self.thinking_options = options;
        self.selected_thinking = Some(selected.into());
        self
    }

    pub fn with_permission_modes(
        mut self,
        options: Vec<String>,
        selected: impl Into<String>,
    ) -> Self {
        self.permission_options = options;
        self.selected_permission = Some(selected.into());
        self
    }

    pub fn with_available_vendors(mut self, vendor_ids: Vec<String>) -> Self {
        self.available_vendor_ids = vendor_ids;
        if !self
            .available_vendor_ids
            .iter()
            .any(|it| it == &self.active_vendor_id)
        {
            self.available_vendor_ids
                .push(self.active_vendor_id.clone());
        }
        self
    }
}

impl Default for VendorUiState {
    fn default() -> Self {
        Self::without_vendor()
    }
}
