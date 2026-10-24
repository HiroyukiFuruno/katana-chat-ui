use super::super::{VendorCapabilityFact, VendorFact, VendorFactRegistry};

pub(super) struct VendorRegistrySnapshot;

impl VendorRegistrySnapshot {
    pub(super) fn render(registry: &VendorFactRegistry) -> String {
        registry
            .facts
            .iter()
            .map(Self::vendor)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn vendor(fact: &VendorFact) -> String {
        [
            format!(
                "vendor={} name={} kind={:?}",
                fact.vendor_id, fact.display_name, fact.connection_kind
            ),
            Self::capability("endpoint", &fact.endpoint),
            Self::capability("model", &fact.model),
            Self::capability("mode", &fact.mode),
            Self::capability("thinking", &fact.thinking),
            Self::capability("permission", &fact.permission),
            Self::capability("tools", &fact.tools),
            Self::capability("web_search", &fact.web_search),
            Self::capability("usage", &fact.usage),
            Self::capability("account_usage", &fact.account_usage),
            Self::capability("attachment", &fact.attachment),
        ]
        .join(" | ")
    }

    fn capability(label: &str, fact: &VendorCapabilityFact) -> String {
        let official_url = match &fact.official_url {
            Some(url) => url.as_str(),
            None => "-",
        };
        format!("{label}={:?}:{official_url}", fact.status)
    }
}
