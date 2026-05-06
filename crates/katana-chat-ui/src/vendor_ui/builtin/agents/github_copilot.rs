use super::{VendorConnectionKind, official, requires_adapter, requires_host};
use crate::VendorFact;

pub(crate) struct GitHubCopilotVendorFact;

impl GitHubCopilotVendorFact {
    pub(crate) fn build() -> VendorFact {
        let reference = official(
            "GitHub Copilot features",
            "https://docs.github.com/en/copilot/get-started/features",
            "Copilot surfaces and agents",
        );
        VendorFact {
            vendor_id: "github-copilot".to_string(),
            display_name: "GitHub Copilot".to_string(),
            connection_kind: VendorConnectionKind::HostExtension,
            references: vec![reference],
            endpoint: requires_adapter("no kcu direct provider"),
            model: requires_adapter("host or extension owns model UI"),
            mode: requires_adapter("host or extension owns mode UI"),
            thinking: requires_adapter("host or extension owns reasoning UI"),
            permission: requires_adapter("host agent owns approval UI"),
            tools: requires_adapter("extension surface required"),
            web_search: requires_adapter("extension surface required"),
            usage: requires_host("GitHub account surface owns usage"),
            account_usage: requires_host("GitHub account surface owns usage"),
            attachment: requires_adapter("extension surface required"),
        }
    }
}
