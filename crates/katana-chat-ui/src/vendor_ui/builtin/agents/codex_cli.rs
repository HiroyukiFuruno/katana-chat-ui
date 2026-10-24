use super::{VendorConnectionKind, official, requires_adapter, requires_host, supported};
use crate::{OfficialReference, VendorCapabilityFact, VendorFact};

pub(crate) struct CodexCliVendorFact;

impl CodexCliVendorFact {
    pub(crate) fn build() -> VendorFact {
        let refs = CodexCliReferences::new();
        VendorFact {
            vendor_id: "codex-cli".to_string(),
            display_name: "Codex CLI".to_string(),
            connection_kind: VendorConnectionKind::AcpAgent,
            references: refs.all(),
            endpoint: requires_adapter("agent login flow owns connection"),
            model: refs.model(),
            mode: requires_adapter("agent session config is adapter-owned"),
            thinking: requires_adapter("reasoning effort comes from Codex config"),
            permission: refs.permission(),
            tools: requires_adapter("tool surface belongs to agent adapter"),
            web_search: refs.web_search(),
            usage: requires_adapter("agent adapter must report usage"),
            account_usage: requires_host("OpenAI account surface owns usage"),
            attachment: refs.attachment(),
        }
    }
}

struct CodexCliReferences {
    cli: OfficialReference,
    options: OfficialReference,
}

impl CodexCliReferences {
    fn new() -> Self {
        Self {
            cli: official(
                "Codex CLI",
                "https://developers.openai.com/codex/cli",
                "local CLI agent",
            ),
            options: official(
                "Codex CLI command line options",
                "https://developers.openai.com/codex/cli#command-line-options",
                "model, approval, sandbox, image input and web search flags",
            ),
        }
    }

    fn all(&self) -> Vec<OfficialReference> {
        vec![self.cli.clone(), self.options.clone()]
    }

    fn model(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.options.clone()],
            "model flag is official",
        )
    }

    fn permission(&self) -> VendorCapabilityFact {
        supported(
            vec![
                "untrusted".to_string(),
                "on-request".to_string(),
                "never".to_string(),
            ],
            vec![self.options.clone()],
            "approval policy flag is official",
        )
    }

    fn web_search(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.options.clone()],
            "web search flag is official",
        )
    }

    fn attachment(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.options.clone()],
            "image input flag is official",
        )
    }
}
