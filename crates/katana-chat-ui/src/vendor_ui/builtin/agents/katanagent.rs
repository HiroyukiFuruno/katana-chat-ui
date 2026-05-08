use super::{VendorConnectionKind, official, requires_adapter, requires_host, supported};
use crate::{OfficialReference, VendorCapabilityFact, VendorFact};

pub(crate) struct KatanAgentVendorFact;

impl KatanAgentVendorFact {
    pub(crate) fn build() -> VendorFact {
        let refs = KatanAgentReferences::new();
        VendorFact {
            vendor_id: "katanagent".to_string(),
            display_name: "KatanAgent".to_string(),
            connection_kind: VendorConnectionKind::AcpAgent,
            references: refs.all(),
            endpoint: requires_adapter("runtime adapter owns local/cloud endpoint"),
            model: refs.model(),
            mode: requires_adapter("agent session mode is adapter-owned"),
            thinking: refs.thinking(),
            permission: refs.permission(),
            tools: refs.tools(),
            web_search: requires_adapter("runtime adapter must expose web fetch support"),
            usage: requires_adapter("runtime adapter must report usage"),
            account_usage: requires_host("provider account surface owns account usage"),
            attachment: refs.attachment(),
        }
    }
}

struct KatanAgentReferences {
    scope: OfficialReference,
    spec: OfficialReference,
}

impl KatanAgentReferences {
    fn new() -> Self {
        Self {
            scope: official(
                "katana-chat-ui v0.1.0 scope",
                "https://github.com/HiroyukiFuruno/katana-chat-ui/blob/main/openspec/v0-1-0-scope.md",
                "KatanAgent is a kcu-owned agent abstraction and Ollama is its initial local runtime",
            ),
            spec: official(
                "katana-chat-ui AI chat UI spec",
                "https://github.com/HiroyukiFuruno/katana-chat-ui/blob/main/docs/chat-ui-spec.ja.md",
                "KatanAgent owns chat, session, output and Markdown file operations",
            ),
        }
    }

    fn all(&self) -> Vec<OfficialReference> {
        vec![self.scope.clone(), self.spec.clone()]
    }

    fn model(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.scope.clone()],
            "model options come from the selected runtime adapter",
        )
    }

    fn thinking(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.spec.clone()],
            "thinking control is available only when the runtime reports options",
        )
    }

    fn permission(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.spec.clone()],
            "permission policy belongs to the agent execution contract",
        )
    }

    fn tools(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.spec.clone()],
            "Markdown file generation and editing are part of the agent surface",
        )
    }

    fn attachment(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.spec.clone()],
            "attachments are passed through the agent request contract",
        )
    }
}
