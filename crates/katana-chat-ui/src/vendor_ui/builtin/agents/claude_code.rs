use super::{VendorConnectionKind, official, requires_adapter, supported, unavailable};
use crate::{OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorFact};

pub(crate) struct ClaudeCodeVendorFact;

impl ClaudeCodeVendorFact {
    pub(crate) fn build() -> VendorFact {
        let refs = ClaudeCodeReferences::new();
        VendorFact {
            vendor_id: "claude-code".to_string(),
            display_name: "Claude Code".to_string(),
            connection_kind: VendorConnectionKind::AcpAgent,
            references: refs.all(),
            endpoint: requires_adapter("agent login flow owns connection"),
            model: refs.model(),
            mode: requires_adapter("agent session config is adapter-owned"),
            thinking: refs.thinking(),
            permission: refs.permission(),
            tools: requires_adapter("tool surface belongs to agent adapter"),
            web_search: unavailable(VendorCapabilityStatus::Unknown, "not fixed for kcu v0.1.0"),
            usage: requires_adapter("agent adapter must report usage"),
            account_usage: requires_adapter("agent login owns account"),
            attachment: requires_adapter("agent adapter must expose support"),
        }
    }
}

struct ClaudeCodeReferences {
    settings: OfficialReference,
    model: OfficialReference,
}

impl ClaudeCodeReferences {
    fn new() -> Self {
        Self {
            settings: official(
                "Claude Code settings",
                "https://code.claude.com/docs/en/settings",
                "settings and permissions",
            ),
            model: official(
                "Claude Code model config",
                "https://code.claude.com/docs/en/model-config",
                "model and effort config",
            ),
        }
    }

    fn all(&self) -> Vec<OfficialReference> {
        vec![self.settings.clone(), self.model.clone()]
    }

    fn model(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.model.clone()],
            "model setting and picker are official",
        )
    }

    fn thinking(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.model.clone()],
            "effort level is official",
        )
    }

    fn permission(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.settings.clone()],
            "permissions settings are official",
        )
    }
}
