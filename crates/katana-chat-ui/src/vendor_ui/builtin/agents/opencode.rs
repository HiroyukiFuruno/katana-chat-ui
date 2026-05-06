use super::{VendorConnectionKind, official, requires_adapter, requires_host, supported};
use crate::{OfficialReference, VendorCapabilityFact, VendorFact};

pub(crate) struct OpenCodeVendorFact;

impl OpenCodeVendorFact {
    pub(crate) fn build() -> VendorFact {
        let refs = OpenCodeReferences::new();
        VendorFact {
            vendor_id: "opencode".to_string(),
            display_name: "OpenCode".to_string(),
            connection_kind: VendorConnectionKind::AcpAgent,
            references: refs.all(),
            endpoint: requires_adapter("agent login flow owns connection"),
            model: refs.model(),
            mode: requires_adapter("agent session config is adapter-owned"),
            thinking: refs.thinking(),
            permission: requires_adapter("agent permission config is adapter-owned"),
            tools: requires_adapter("tool surface belongs to agent adapter"),
            web_search: requires_adapter("agent adapter must report web search"),
            usage: requires_adapter("agent adapter must report usage"),
            account_usage: requires_host("provider account surface owns usage"),
            attachment: refs.attachment(),
        }
    }
}

struct OpenCodeReferences {
    cli: OfficialReference,
}

impl OpenCodeReferences {
    fn new() -> Self {
        Self {
            cli: official(
                "OpenCode CLI",
                "https://opencode.ai/docs/cli/",
                "models, run, thinking and file flags",
            ),
        }
    }

    fn all(&self) -> Vec<OfficialReference> {
        vec![self.cli.clone()]
    }

    fn model(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.cli.clone()],
            "models command and run --model are official",
        )
    }

    fn thinking(&self) -> VendorCapabilityFact {
        supported(
            vec!["false".to_string(), "true".to_string()],
            vec![self.cli.clone()],
            "run --thinking is official",
        )
    }

    fn attachment(&self) -> VendorCapabilityFact {
        supported(Vec::new(), vec![self.cli.clone()], "run --file is official")
    }
}
