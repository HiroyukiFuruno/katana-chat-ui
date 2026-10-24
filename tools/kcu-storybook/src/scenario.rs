use katana_chat_ui::{ChatSession, ContextUsageSnapshot, VendorUiState};

pub(crate) const STORYBOOK_HISTORY_ID: &str = "storybook-reference";
const VENDORS: &[&str] = &["codex-cli", "claude-code", "github-copilot", "katanagent"];
const SCROLL_FIXTURE_SECTIONS: &[&str] = &[
    "入力、送信、固定応答はproviderへ接続せずKUCの共通runtimeだけを通ります。",
    "vendorを切り替えると、そのvendorが公開するcontrolだけが表示されます。",
    "実行可能なmodelだけを選択肢にし、未導入のOllama modelをfixtureで補いません。",
    "履歴を開いてfixtureを選ぶと、初期の会話状態へ戻ります。",
    "添付、Markdown、リンク、code spanは通常のKCU表示経路で描画されます。",
    "長文fixtureはthread scrollとcomposer固定表示を確認するために用意しています。",
    "この画面は認証、課金契約、model downloadを行わない確認用shellです。",
    "実providerとの通信はhost application側で明示的に設定した場合だけ開始されます。",
];

pub(crate) fn seeded_session() -> Result<ChatSession, String> {
    let mut session = ChatSession::new();
    session.set_title("KCU Storybook");
    session.set_provider_configured("Storybook fixture");
    session.set_vendor_ui_state(vendor_state("claude-code")?);
    session.set_context_usage(ContextUsageSnapshot::new(196_608, 262_144));

    session
        .draft_mut()
        .set_text("KCU Storybook では何を確認できますか？");
    session.submit_draft().map_err(|error| error.to_string())?;
    session
        .start_assistant_stream("# KCU Storybook\n\n公開 KUC の共通 runtime を通る対話確認面です。")
        .map_err(|error| error.to_string())?;
    session
        .append_assistant_chunk(
            "\n\n- provider を使わない fixture\n- 日本語入力と Command+Enter\n- provider / model 選択\n- 履歴、新規チャット、Markdown と scroll",
        )
        .map_err(|error| error.to_string())?;
    for (index, section) in SCROLL_FIXTURE_SECTIONS.iter().enumerate() {
        session
            .append_assistant_chunk(format!("\n\n## Scroll fixture {}\n\n{section}", index + 1))
            .map_err(|error| error.to_string())?;
    }
    session
        .finish_assistant_message()
        .map_err(|error| error.to_string())?;
    Ok(session)
}

pub(crate) fn vendor_state(vendor_id: &str) -> Result<VendorUiState, String> {
    if !VENDORS.contains(&vendor_id) {
        return Err(format!("unsupported Storybook vendor: {vendor_id}"));
    }
    let available = VENDORS.iter().map(ToString::to_string).collect();
    let state = match vendor_id {
        "codex-cli" => VendorUiState::for_vendor(vendor_id)
            .with_available_vendors(available)
            .with_models(vec!["gpt-5.6".into(), "gpt-5.5".into()], "gpt-5.6")
            .with_thinking(vec!["medium".into(), "high".into()], "medium"),
        "claude-code" => VendorUiState::for_vendor(vendor_id)
            .with_available_vendors(available)
            .with_thinking(vec!["default".into(), "high".into()], "default"),
        "github-copilot" => VendorUiState::for_vendor(vendor_id).with_available_vendors(available),
        "katanagent" => VendorUiState::for_vendor(vendor_id).with_available_vendors(available),
        _ => return Err(format!("unsupported Storybook vendor: {vendor_id}")),
    };
    Ok(state)
}

pub(crate) fn provider_label(vendor_id: &str) -> Result<&'static str, String> {
    match vendor_id {
        "codex-cli" => Ok("Codex CLI"),
        "claude-code" => Ok("Claude Code"),
        "github-copilot" => Ok("GitHub Copilot"),
        "katanagent" => Ok("KatanAgent"),
        _ => Err(format!("unsupported Storybook vendor: {vendor_id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{seeded_session, vendor_state};
    use katana_chat_ui::ChatUiSurface;

    #[test]
    fn fixture_uses_all_required_multi_vendor_options() -> Result<(), String> {
        let surface = ChatUiSurface::from_render_model(&seeded_session()?.render_model());
        let ids: Vec<_> = surface
            .vendor_bar
            .vendor_options
            .iter()
            .map(|option| option.id.as_str())
            .collect();
        assert_eq!(
            ids,
            ["codex-cli", "claude-code", "github-copilot", "katanagent"]
        );
        Ok(())
    }

    #[test]
    fn unavailable_vendors_do_not_invent_local_models() -> Result<(), String> {
        let state = vendor_state("claude-code")?;
        assert!(state.selected_model.is_none());
        assert!(state.model_options.is_empty());
        let state = vendor_state("katanagent")?;
        assert!(state.selected_model.is_none());
        assert!(state.model_options.is_empty());
        Ok(())
    }

    #[test]
    fn fixture_has_enough_sections_to_exercise_thread_scrolling() {
        assert!(super::SCROLL_FIXTURE_SECTIONS.len() >= 8);
        assert!(
            super::SCROLL_FIXTURE_SECTIONS
                .iter()
                .all(|section| !section.is_empty())
        );
    }
}
