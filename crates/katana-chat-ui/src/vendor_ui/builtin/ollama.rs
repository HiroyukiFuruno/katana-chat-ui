use crate::{
    OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorConnectionKind,
    VendorFact,
};

const REVIEWED_ON: &str = "2026-05-06";

pub(super) struct OllamaVendorFact;

impl OllamaVendorFact {
    pub(super) fn build() -> VendorFact {
        let refs = OllamaReferences::new();
        VendorFact {
            vendor_id: "ollama".to_string(),
            display_name: "Ollama local".to_string(),
            connection_kind: VendorConnectionKind::LocalDirect,
            references: refs.all(),
            endpoint: refs.endpoint(),
            model: refs.model(),
            mode: refs.mode(),
            thinking: refs.thinking(),
            permission: refs.permission(),
            tools: refs.tools(),
            web_search: refs.web_search(),
            usage: refs.usage(),
            account_usage: unavailable(VendorCapabilityStatus::Unsupported, "no account usage API"),
            attachment: refs.attachment(),
        }
    }
}

struct OllamaReferences {
    api: OfficialReference,
    chat: OfficialReference,
    tags: OfficialReference,
}

impl OllamaReferences {
    fn new() -> Self {
        Self {
            api: official(
                "Ollama API",
                "https://docs.ollama.com/api/introduction",
                "local API base URL",
            ),
            chat: official(
                "Ollama Chat API",
                "https://docs.ollama.com/api/chat",
                "chat, tools, think, token counts",
            ),
            tags: official(
                "Ollama model list",
                "https://docs.ollama.com/api/tags",
                "runtime model list",
            ),
        }
    }

    fn all(&self) -> Vec<OfficialReference> {
        vec![self.api.clone(), self.chat.clone(), self.tags.clone()]
    }

    fn endpoint(&self) -> VendorCapabilityFact {
        supported(
            vec!["http://localhost:11434".to_string()],
            vec![self.api.clone()],
            "local endpoint",
        )
    }

    fn model(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.tags.clone()],
            "models come from /api/tags",
        )
    }

    fn mode(&self) -> VendorCapabilityFact {
        unavailable(
            VendorCapabilityStatus::Unsupported,
            "no official chat/agent mode selector",
        )
    }

    fn thinking(&self) -> VendorCapabilityFact {
        supported(
            thinking_options(),
            vec![self.chat.clone()],
            "think accepts boolean or low/medium/high",
        )
    }

    fn tools(&self) -> VendorCapabilityFact {
        unavailable(
            VendorCapabilityStatus::RequiresAdapter,
            "raw tool schema is not an agent execution or approval surface",
        )
    }

    fn permission(&self) -> VendorCapabilityFact {
        unavailable(
            VendorCapabilityStatus::Unsupported,
            "no permission mode in Ollama API",
        )
    }

    fn web_search(&self) -> VendorCapabilityFact {
        unavailable(
            VendorCapabilityStatus::Unsupported,
            "no built-in web search control",
        )
    }

    fn usage(&self) -> VendorCapabilityFact {
        supported(
            Vec::new(),
            vec![self.chat.clone()],
            "response exposes prompt and eval token counts",
        )
    }

    fn attachment(&self) -> VendorCapabilityFact {
        unavailable(
            VendorCapabilityStatus::Unknown,
            "model-specific; not inferred by kcu",
        )
    }
}

fn thinking_options() -> Vec<String> {
    vec![
        "false".to_string(),
        "low".to_string(),
        "medium".to_string(),
        "high".to_string(),
    ]
}

fn official(title: &str, url: &str, note: &str) -> OfficialReference {
    OfficialReference::new(title, url, REVIEWED_ON, note)
}

fn supported(
    options: Vec<String>,
    references: Vec<OfficialReference>,
    note: &str,
) -> VendorCapabilityFact {
    VendorCapabilityFact::supported(options, references, note)
}

fn unavailable(status: VendorCapabilityStatus, note: &str) -> VendorCapabilityFact {
    VendorCapabilityFact::unavailable(status, note)
}
