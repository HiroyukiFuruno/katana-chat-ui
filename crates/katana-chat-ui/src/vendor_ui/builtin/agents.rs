mod claude_code;
mod codex_cli;
mod github_copilot;
mod opencode;

use crate::{
    OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorConnectionKind,
};

pub(super) use claude_code::ClaudeCodeVendorFact;
pub(super) use codex_cli::CodexCliVendorFact;
pub(super) use github_copilot::GitHubCopilotVendorFact;
pub(super) use opencode::OpenCodeVendorFact;

const REVIEWED_ON: &str = "2026-05-06";

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

fn requires_adapter(note: &str) -> VendorCapabilityFact {
    unavailable(VendorCapabilityStatus::RequiresAdapter, note)
}

fn requires_host(note: &str) -> VendorCapabilityFact {
    unavailable(VendorCapabilityStatus::RequiresHost, note)
}

fn unavailable(status: VendorCapabilityStatus, note: &str) -> VendorCapabilityFact {
    VendorCapabilityFact::unavailable(status, note)
}
