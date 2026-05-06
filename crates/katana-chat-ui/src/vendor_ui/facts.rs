use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VendorConnectionKind {
    LocalDirect,
    OpenAiCompatibleDirect,
    CloudDirect,
    AcpAgent,
    HostExtension,
    UnsupportedDirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VendorCapabilityStatus {
    Supported,
    Unsupported,
    RequiresAdapter,
    RequiresHost,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficialReference {
    pub title: String,
    pub url: String,
    pub reviewed_on: String,
    pub note: String,
}

impl OfficialReference {
    pub fn new(
        title: impl Into<String>,
        url: impl Into<String>,
        reviewed_on: impl Into<String>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            reviewed_on: reviewed_on.into(),
            note: note.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorCapabilityFact {
    pub status: VendorCapabilityStatus,
    pub options: Vec<String>,
    pub official_url: Option<String>,
    pub references: Vec<OfficialReference>,
    pub note: String,
}

impl VendorCapabilityFact {
    pub fn supported(
        options: Vec<String>,
        references: Vec<OfficialReference>,
        note: impl Into<String>,
    ) -> Self {
        let official_url = references.first().map(|it| it.url.clone());
        Self {
            status: VendorCapabilityStatus::Supported,
            options,
            official_url,
            references,
            note: note.into(),
        }
    }

    pub fn unavailable(status: VendorCapabilityStatus, note: impl Into<String>) -> Self {
        Self {
            status,
            options: Vec::new(),
            official_url: None,
            references: Vec::new(),
            note: note.into(),
        }
    }

    pub fn supported_by_official_reference(&self) -> bool {
        self.status == VendorCapabilityStatus::Supported && self.official_url.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorFact {
    pub vendor_id: String,
    pub display_name: String,
    pub connection_kind: VendorConnectionKind,
    pub references: Vec<OfficialReference>,
    pub endpoint: VendorCapabilityFact,
    pub model: VendorCapabilityFact,
    pub mode: VendorCapabilityFact,
    pub thinking: VendorCapabilityFact,
    pub permission: VendorCapabilityFact,
    pub tools: VendorCapabilityFact,
    pub web_search: VendorCapabilityFact,
    pub usage: VendorCapabilityFact,
    pub account_usage: VendorCapabilityFact,
    pub attachment: VendorCapabilityFact,
}

impl VendorFact {
    pub fn validate(&self) -> Result<(), VendorFactValidationError> {
        self.validate_capability("endpoint", &self.endpoint)?;
        self.validate_capability("model", &self.model)?;
        self.validate_capability("mode", &self.mode)?;
        self.validate_capability("thinking", &self.thinking)?;
        self.validate_capability("permission", &self.permission)?;
        self.validate_capability("tools", &self.tools)?;
        self.validate_capability("web_search", &self.web_search)?;
        self.validate_capability("usage", &self.usage)?;
        self.validate_capability("account_usage", &self.account_usage)?;
        self.validate_capability("attachment", &self.attachment)
    }

    fn validate_capability(
        &self,
        capability: &'static str,
        fact: &VendorCapabilityFact,
    ) -> Result<(), VendorFactValidationError> {
        if fact.status == VendorCapabilityStatus::Supported && fact.official_url.is_none() {
            return Err(VendorFactValidationError::MissingOfficialReference {
                vendor_id: self.vendor_id.clone(),
                capability,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorFactRegistry {
    pub facts: Vec<VendorFact>,
}

impl VendorFactRegistry {
    pub fn new(facts: Vec<VendorFact>) -> Self {
        Self { facts }
    }

    pub fn builtin() -> Self {
        crate::vendor_ui::builtin::BuiltinVendorFacts::registry()
    }

    pub fn get(&self, vendor_id: &str) -> Option<&VendorFact> {
        self.facts.iter().find(|it| it.vendor_id == vendor_id)
    }

    pub fn validate(&self) -> Result<(), VendorFactValidationError> {
        for fact in &self.facts {
            fact.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VendorFactValidationError {
    MissingOfficialReference {
        vendor_id: String,
        capability: &'static str,
    },
}

impl Default for VendorFactRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}
