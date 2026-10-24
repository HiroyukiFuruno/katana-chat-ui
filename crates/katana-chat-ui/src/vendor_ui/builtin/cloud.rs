use crate::{
    OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorConnectionKind,
    VendorFact,
};

const REVIEWED_ON: &str = "2026-05-06";

pub(super) struct OpenAiCompatibleVendorFact;
pub(super) struct AnthropicVendorFact;
pub(super) struct VertexAiVendorFact;
pub(super) struct BedrockVendorFact;

impl OpenAiCompatibleVendorFact {
    pub(super) fn build() -> VendorFact {
        let refs = vec![
            official(
                "OpenAI models",
                "https://developers.openai.com/api/docs/models",
                "model capabilities",
            ),
            official(
                "OpenAI tools",
                "https://developers.openai.com/api/docs/guides/tools",
                "tool controls",
            ),
        ];
        cloud_fact(
            "openai-compatible",
            "OpenAI-compatible",
            VendorConnectionKind::OpenAiCompatibleDirect,
            refs,
        )
    }
}

impl AnthropicVendorFact {
    pub(super) fn build() -> VendorFact {
        let refs = vec![official(
            "Claude API tool use",
            "https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools",
            "tool use",
        )];
        cloud_fact(
            "anthropic-api",
            "Anthropic API",
            VendorConnectionKind::CloudDirect,
            refs,
        )
    }
}

impl VertexAiVendorFact {
    pub(super) fn build() -> VendorFact {
        let refs = vec![official(
            "Vertex AI models",
            "https://docs.cloud.google.com/vertex-ai/generative-ai/docs/models",
            "model catalog",
        )];
        cloud_fact(
            "vertex-ai",
            "Vertex AI",
            VendorConnectionKind::CloudDirect,
            refs,
        )
    }
}

impl BedrockVendorFact {
    pub(super) fn build() -> VendorFact {
        let refs = vec![official(
            "Amazon Bedrock Converse API",
            "https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-call.html",
            "Converse and ConverseStream",
        )];
        cloud_fact(
            "bedrock",
            "Amazon Bedrock",
            VendorConnectionKind::CloudDirect,
            refs,
        )
    }
}

fn cloud_fact(
    id: &str,
    name: &str,
    kind: VendorConnectionKind,
    references: Vec<OfficialReference>,
) -> VendorFact {
    VendorFact {
        vendor_id: id.to_string(),
        display_name: name.to_string(),
        connection_kind: kind,
        references,
        endpoint: requires_host(),
        model: requires_host(),
        mode: requires_host(),
        thinking: requires_host(),
        permission: requires_host(),
        tools: requires_host(),
        web_search: requires_host(),
        usage: requires_host(),
        account_usage: requires_host(),
        attachment: requires_host(),
    }
}

fn official(title: &str, url: &str, note: &str) -> OfficialReference {
    OfficialReference::new(title, url, REVIEWED_ON, note)
}

fn requires_host() -> VendorCapabilityFact {
    VendorCapabilityFact::unavailable(
        VendorCapabilityStatus::RequiresHost,
        "secure connector is required",
    )
}
