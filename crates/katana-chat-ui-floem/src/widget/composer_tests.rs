use super::composer::{ComposerDraftSubmitter, ComposerEditorState, ComposerSubmitGate};
use floem::prelude::*;
use katana_chat_ui::{ChatSession, ChatUiComposerSurface, ChatUiSurface};

const COMPOSER_SOURCE: &str = include_str!("composer.rs");

#[test]
fn composer_uses_shared_chat_body_width_contract() {
    assert!(COMPOSER_SOURCE.contains(".width_full()"));
    assert!(COMPOSER_SOURCE.contains("max_width(styles::CHAT_BODY_MAX_WIDTH)"));
    assert!(COMPOSER_SOURCE.contains(".justify_center()"));
}

#[test]
fn composer_controls_keep_primary_action_inside_composer() {
    let controls_source = include_str!("composer_controls.rs");
    let vendor_source = include_str!("vendor_controls.rs");

    assert!(controls_source.contains("fn action_row"));
    assert!(controls_source.contains("Self::center(surfaces.vendor"));
    assert!(controls_source.contains(".justify_center()"));
    assert!(controls_source.contains(".items_center()"));
    assert!(controls_source.contains(".flex_shrink(0.0)"));
    assert!(vendor_source.contains("FlexWrap::Wrap"));
    assert!(vendor_source.contains("v_stack(("));
    assert!(!vendor_source.contains("Self::vendor_dropdown"));
}

#[test]
fn composer_control_layout_keeps_usage_next_to_primary_action() {
    let controls_source = include_str!("composer_controls.rs");
    let action_source = include_str!("action_button.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap_or("");

    assert!(controls_source.contains("FloemComposerUsageView::render(surfaces.usage)"));
    assert!(controls_source.contains("Self::right(surfaces.composer"));
    assert!(
        controls_source
            .find("FloemComposerUsageView::render(surfaces.usage)")
            .unwrap_or(usize::MAX)
            < controls_source
                .find("Self::right(surfaces.composer")
                .unwrap_or(0)
    );
    assert!(action_source.contains(".background(Color::TRANSPARENT)"));
    assert!(!action_source.contains(".border(1.0)"));
}

#[test]
fn provider_selector_lives_on_toolbar_not_composer_controls() {
    let toolbar_source = include_str!("toolbar.rs");
    let selector_source = include_str!("provider_icon_selector.rs");

    assert!(toolbar_source.contains("FloemProviderIconSelector::render"));
    assert!(toolbar_source.contains("chrome.provider_icon"));
    assert!(selector_source.contains("fn provider_dropdown_button"));
}

#[test]
fn composer_renders_slash_launcher_inside_composer_frame() {
    assert!(COMPOSER_SOURCE.contains("fn slash_launcher"));
    assert!(COMPOSER_SOURCE.contains("surface.get().composer.slash_launcher"));
    assert!(COMPOSER_SOURCE.contains("slash_launcher_panel"));
}

#[test]
fn toolbar_exposes_new_chat_and_history_actions() {
    let toolbar_source = include_str!("toolbar.rs");

    assert!(toolbar_source.contains("toolbar_actions"));
    assert!(toolbar_source.contains("chrome.new_chat"));
    assert!(toolbar_source.contains("chrome.history"));
    assert!(!toolbar_source.contains("chrome.settings"));
}

#[test]
fn editor_initial_text_keeps_live_draft_after_surface_refresh() {
    let composer = composer_surface("");
    let draft = RwSignal::new("こんにちは".to_string());

    assert_eq!(
        ComposerEditorState::initial_text(&composer, draft),
        "こんにちは"
    );
}

#[test]
fn editor_initial_text_uses_surface_text_when_draft_is_empty() {
    let composer = composer_surface("surface draft");
    let draft = RwSignal::new(String::new());

    assert_eq!(
        ComposerEditorState::initial_text(&composer, draft),
        "surface draft"
    );
}

#[test]
fn submitter_sends_non_empty_live_draft() {
    let draft = RwSignal::new("こんにちは".to_string());
    let submitted = RwSignal::new(String::new());

    ComposerDraftSubmitter::submit(draft, move |text| submitted.set(text));

    assert_eq!(submitted.get_untracked(), "こんにちは");
    assert_eq!(draft.get_untracked(), "");
}

#[test]
fn submitter_ignores_blank_draft() {
    let draft = RwSignal::new("   ".to_string());
    let submitted = RwSignal::new(false);

    ComposerDraftSubmitter::submit(draft, move |_| submitted.set(true));

    assert!(!submitted.get_untracked());
}

#[test]
fn submit_gate_allows_attachment_only_when_provider_is_ready() {
    let mut composer = composer_surface("");
    composer
        .attachments
        .push(katana_chat_ui::Attachment::text("sample.md", "# sample"));

    assert!(ComposerSubmitGate::can_submit(&composer, ""));
}

#[test]
fn submitter_clears_attachment_only_draft_after_submit() {
    let mut composer = composer_surface("");
    composer
        .attachments
        .push(katana_chat_ui::Attachment::text("sample.md", "# sample"));
    let draft = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);

    ComposerDraftSubmitter::submit_allowed(&composer, draft, move |_| submitted.set(true));

    assert!(submitted.get_untracked());
    assert_eq!(draft.get_untracked(), "");
}

#[test]
fn submit_gate_rejects_text_when_provider_is_missing() {
    let mut composer = composer_surface("");
    composer.send_available = false;

    assert!(!ComposerSubmitGate::can_submit(&composer, "こんにちは"));
}

#[test]
fn submit_gate_rejects_submit_while_stop_is_active() {
    let mut composer = composer_surface("");
    composer.stop_enabled = true;

    assert!(!ComposerSubmitGate::can_submit(&composer, "こんにちは"));
}

#[test]
fn attach_button_stays_enabled_when_provider_is_missing() {
    let session = ChatSession::new();
    let composer = ChatUiSurface::from_render_model(&session.render_model()).composer;

    assert!(composer.attach.enabled);
}

fn composer_surface(text: &str) -> ChatUiComposerSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text(text);
    ChatUiSurface::from_render_model(&session.render_model()).composer
}
