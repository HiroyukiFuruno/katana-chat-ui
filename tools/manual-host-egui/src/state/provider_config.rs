use katana_chat_ui::{ChatSession, VendorUiState};

pub(crate) fn configure_manual_provider(session: &mut ChatSession) {
    session.set_title("katana-chat-ui");
    session.set_provider_configured("Claude Code");
    session.set_vendor_ui_state(manual_vendor_state("claude-code"));
}

pub(crate) fn manual_vendor_state(vendor_id: &str) -> VendorUiState {
    let vendor_ids = vec![
        "claude-code".to_string(),
        "codex-cli".to_string(),
        "github-copilot".to_string(),
        "opencode".to_string(),
    ];
    match vendor_id {
        "codex-cli" => codex_cli_state(vendor_ids),
        "github-copilot" => github_copilot_state(vendor_ids),
        "opencode" => opencode_state(vendor_ids),
        _ => claude_code_state(vendor_ids),
    }
}

pub(crate) fn provider_label(vendor_id: &str) -> &'static str {
    match vendor_id {
        "codex-cli" => "Codex CLI",
        "github-copilot" => "GitHub Copilot",
        "opencode" => "OpenCode",
        _ => "Claude Code",
    }
}

fn claude_code_state(vendor_ids: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("claude-code")
        .with_available_vendors(vendor_ids)
        .with_models(
            vec![
                "claude-sonnet-4-6".to_string(),
                "claude-haiku-4-5".to_string(),
            ],
            "claude-sonnet-4-6",
        )
        .with_thinking(
            vec![
                "default".to_string(),
                "low".to_string(),
                "medium".to_string(),
                "high".to_string(),
                "xhigh".to_string(),
                "max".to_string(),
            ],
            "default",
        )
        .with_permission_modes(
            vec![
                "default".to_string(),
                "auto".to_string(),
                "plan".to_string(),
            ],
            "default",
        )
}

fn codex_cli_state(vendor_ids: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("codex-cli")
        .with_available_vendors(vendor_ids)
        .with_models(
            vec!["gpt-5.5".to_string(), "gpt-5.4".to_string()],
            "gpt-5.5",
        )
        .with_permission_modes(
            vec![
                "untrusted".to_string(),
                "on-request".to_string(),
                "never".to_string(),
            ],
            "on-request",
        )
}

fn github_copilot_state(vendor_ids: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("github-copilot")
        .with_available_vendors(vendor_ids)
        .with_models(vec!["copilot".to_string()], "copilot")
}

fn opencode_state(vendor_ids: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("opencode")
        .with_available_vendors(vendor_ids)
        .with_models(vec!["opencode/default".to_string()], "opencode/default")
        .with_thinking(vec!["false".to_string(), "true".to_string()], "false")
}
