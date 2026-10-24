use serde::{Deserialize, Serialize};

const DEFAULT_VENDOR_ID: &str = "default";
const DEFAULT_VENDOR_LABEL: &str = "Default";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorUiProfile {
    pub vendor_id: String,
    pub display_name: String,
    pub capabilities: VendorUiCapabilities,
}

impl VendorUiProfile {
    pub fn new(
        vendor_id: impl Into<String>,
        display_name: impl Into<String>,
        capabilities: VendorUiCapabilities,
    ) -> Self {
        Self {
            vendor_id: vendor_id.into(),
            display_name: display_name.into(),
            capabilities,
        }
    }
}

impl Default for VendorUiProfile {
    fn default() -> Self {
        Self::new(
            DEFAULT_VENDOR_ID,
            DEFAULT_VENDOR_LABEL,
            VendorUiCapabilities::default(),
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorUiCapabilities {
    pub model_labels: Vec<String>,
    pub mode_labels: Vec<String>,
    pub thinking_labels: Vec<String>,
    pub permission_mode_labels: Vec<String>,
    pub tool_approval: bool,
    pub web_search: bool,
}

impl VendorUiCapabilities {
    pub fn with_models(mut self, labels: Vec<String>) -> Self {
        self.model_labels = labels;
        self
    }

    pub fn with_modes(mut self, labels: Vec<String>) -> Self {
        self.mode_labels = labels;
        self
    }

    pub fn with_thinking_levels(mut self, labels: Vec<String>) -> Self {
        self.thinking_labels = labels;
        self
    }

    pub fn with_permission_modes(mut self, labels: Vec<String>) -> Self {
        self.permission_mode_labels = labels;
        self
    }

    pub fn with_tool_approval(mut self) -> Self {
        self.tool_approval = true;
        self
    }

    pub fn with_web_search(mut self) -> Self {
        self.web_search = true;
        self
    }
}
