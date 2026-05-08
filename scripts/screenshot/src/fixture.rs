use crate::request::{Scenario, ScenarioKind};
use anyhow::Result;
use katana_chat_ui::{
    ChatOutputKind, ChatSession, ChatUiSurface, ContextUsageSnapshot, DiffCandidateOutput,
    ThinkingLog, VendorUiState,
};

pub fn surface(scenario: &Scenario) -> Result<ChatUiSurface> {
    let mut session = ChatSession::new();
    session.set_title(scenario.title.clone());
    session.set_provider_configured(provider_label(&scenario.provider_id));
    session.set_vendor_ui_state(vendor_state(&scenario.provider_id));
    session.set_context_usage(ContextUsageSnapshot::new(196_608, 262_144));
    session.set_debug_enabled(debug_enabled(scenario));

    add_scenario_messages(&mut session, scenario.kind)?;
    session.draft_mut().set_text(scenario.draft.clone());
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn debug_enabled(scenario: &Scenario) -> bool {
    scenario.debug || matches!(scenario.kind, ScenarioKind::OutputDebug)
}

fn add_scenario_messages(session: &mut ChatSession, kind: ScenarioKind) -> Result<()> {
    match kind {
        ScenarioKind::Empty => Ok(()),
        ScenarioKind::Conversation => add_conversation(session),
        ScenarioKind::Thinking => add_thinking(session),
        ScenarioKind::OutputDebug => add_conversation(session),
    }
}

fn add_conversation(session: &mut ChatSession) -> Result<()> {
    session.draft_mut().set_text("こんにちは");
    let user_id = session.submit_draft()?;
    let assistant_id = session.start_assistant_stream("mock provider response")?;
    session.append_assistant_chunk("\n\n- provider is mocked\n- output is captured headlessly")?;
    session.add_output(
        assistant_id,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/floem-generated.md",
            "--- a/tmp/floem-generated.md\n+++ b/tmp/floem-generated.md\n@@ -1 +1 @@\n-before\n+after",
        )),
    )?;
    session.finish_assistant_message()?;
    add_follow_up(session, user_id)
}

fn add_follow_up(session: &mut ChatSession, _previous_user_id: u64) -> Result<()> {
    session.draft_mut().set_text("さようなら");
    session.submit_draft()?;
    let assistant_id = session.start_assistant_stream("mock provider response")?;
    session.append_assistant_chunk("\n\n```rust\nfn main() {}\n```")?;
    session.finish_assistant_message()?;
    session.add_output(
        assistant_id,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "src/lib.rs",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-before\n+after",
        )),
    )?;
    Ok(())
}

fn add_thinking(session: &mut ChatSession) -> Result<()> {
    session.draft_mut().set_text("ログを要約して");
    session.submit_draft()?;
    session.start_assistant_stream_with_thinking(
        "",
        ThinkingLog::running(
            "Thinking",
            vec![
                "入力内容と添付の有無を確認しています。".to_string(),
                "回答方針を組み立てています。".to_string(),
            ],
        ),
    )?;
    Ok(())
}

fn vendor_state(provider_id: &str) -> VendorUiState {
    let available = vec![
        "claude-code".to_string(),
        "codex-cli".to_string(),
        "github-copilot".to_string(),
        "opencode".to_string(),
    ];
    match provider_id {
        "codex-cli" => codex_state(available),
        "github-copilot" => github_copilot_state(available),
        "opencode" => opencode_state(available),
        _ => claude_code_state(available),
    }
}

fn claude_code_state(available: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("claude-code")
        .with_available_vendors(available)
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
            ],
            "default",
        )
        .with_permission_modes(vec!["default".to_string(), "auto".to_string()], "default")
}

fn codex_state(available: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("codex-cli")
        .with_available_vendors(available)
        .with_models(
            vec!["gpt-5.5".to_string(), "gpt-5.4".to_string()],
            "gpt-5.5",
        )
        .with_thinking(
            vec![
                "medium".to_string(),
                "high".to_string(),
                "xhigh".to_string(),
            ],
            "medium",
        )
        .with_permission_modes(vec!["ask".to_string(), "auto".to_string()], "ask")
}

fn github_copilot_state(available: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("github-copilot").with_available_vendors(available)
}

fn opencode_state(available: Vec<String>) -> VendorUiState {
    VendorUiState::for_vendor("opencode").with_available_vendors(available)
}

fn provider_label(provider_id: &str) -> &'static str {
    match provider_id {
        "codex-cli" => "Codex CLI",
        "github-copilot" => "GitHub Copilot",
        "opencode" => "OpenCode",
        _ => "Claude Code",
    }
}

#[cfg(test)]
mod tests {
    use super::surface;
    use crate::request::Scenario;

    #[test]
    fn default_fixture_uses_agent_provider_instead_of_ollama() -> anyhow::Result<()> {
        let surface = surface(&Scenario::default())?;

        assert_eq!(surface.vendor_bar.active_vendor_id, "claude-code");
        assert_ne!(surface.vendor_bar.active_vendor_id, "ollama");
        assert!(surface.message_list.messages.len() >= 2);
        Ok(())
    }

    #[test]
    fn thinking_fixture_keeps_answer_body_empty_while_running() -> anyhow::Result<()> {
        let surface = surface(&Scenario {
            kind: crate::request::ScenarioKind::Thinking,
            ..Scenario::default()
        })?;
        let assistant = surface
            .messages
            .iter()
            .find(|it| it.thinking.is_some())
            .ok_or_else(|| anyhow::anyhow!("thinking message must exist"))?;

        assert!(assistant.body.trim().is_empty());
        assert!(assistant.thinking.as_ref().is_some_and(|it| it.expanded));
        Ok(())
    }

    #[test]
    fn output_debug_fixture_enables_debug_handoff() -> anyhow::Result<()> {
        let surface = surface(&Scenario {
            kind: crate::request::ScenarioKind::OutputDebug,
            debug: false,
            ..Scenario::default()
        })?;

        assert!(surface.debug.enabled);
        assert!(surface.debug.output_handoff_text.contains("\"kind\""));
        Ok(())
    }
}
