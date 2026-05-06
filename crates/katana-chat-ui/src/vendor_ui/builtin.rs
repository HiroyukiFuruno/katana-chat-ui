mod agents;
mod cloud;
mod ollama;

use super::VendorFactRegistry;
use agents::{
    ClaudeCodeVendorFact, CodexCliVendorFact, GitHubCopilotVendorFact, OpenCodeVendorFact,
};
use cloud::{
    AnthropicVendorFact, BedrockVendorFact, OpenAiCompatibleVendorFact, VertexAiVendorFact,
};
use ollama::OllamaVendorFact;

pub(super) struct BuiltinVendorFacts;

impl BuiltinVendorFacts {
    pub(super) fn registry() -> VendorFactRegistry {
        VendorFactRegistry::new(vec![
            OllamaVendorFact::build(),
            ClaudeCodeVendorFact::build(),
            CodexCliVendorFact::build(),
            GitHubCopilotVendorFact::build(),
            OpenCodeVendorFact::build(),
            OpenAiCompatibleVendorFact::build(),
            AnthropicVendorFact::build(),
            VertexAiVendorFact::build(),
            BedrockVendorFact::build(),
        ])
    }
}
