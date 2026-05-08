const FLOEM_TOOLBAR: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/toolbar.rs");
const FLOEM_SELECTOR: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/provider_icon_selector.rs");
const FLOEM_COMPOSER: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/composer_controls.rs");
const EGUI_VIEW: &str = include_str!("../../../crates/katana-chat-ui-egui/src/view.rs");
const EGUI_COMPOSER: &str = include_str!("../../../crates/katana-chat-ui-egui/src/composer.rs");
const GPUI_HEADER: &str = include_str!("../../../crates/katana-chat-ui-gpui/src/view/header.rs");
const GPUI_COMPOSER: &str =
    include_str!("../../../crates/katana-chat-ui-gpui/src/view/composer.rs");
const FLOEM_COMPOSER_VIEW: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/composer.rs");
const FLOEM_ROOT: &str = include_str!("../../../crates/katana-chat-ui-floem/src/widget/root.rs");
const FLOEM_THREAD: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/thread.rs");
const FLOEM_COMPOSER_EDITOR_TESTS: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/composer/editor/tests.rs");
const HOST_E2E_APP: &str = include_str!("../src/main.rs");
const SCREENSHOT_REQUEST: &str =
    include_str!("../../../scripts/screenshot/examples/standard-chat.json");
const SCREENSHOT_EMPTY_REQUEST: &str =
    include_str!("../../../scripts/screenshot/examples/empty-chat.json");
const SCREENSHOT_THINKING_REQUEST: &str =
    include_str!("../../../scripts/screenshot/examples/thinking-chat.json");
const SCREENSHOT_NARROW_REQUEST: &str =
    include_str!("../../../scripts/screenshot/examples/narrow-chat.json");
const SCREENSHOT_TALL_REQUEST: &str =
    include_str!("../../../scripts/screenshot/examples/tall-chat.json");

#[test]
fn adapters_use_shared_header_provider_contract() {
    assert_source_contains("floem toolbar", FLOEM_TOOLBAR, "chrome.provider_icon");
    assert_source_contains("floem selector", FLOEM_SELECTOR, "active_vendor_label");
    assert_source_contains("egui header", EGUI_VIEW, "provider_icon(ui, surface)");
    assert_source_contains(
        "egui header",
        EGUI_VIEW,
        "provider_selector(ui, surface, controller)",
    );
    assert_source_contains("gpui header", GPUI_HEADER, "Self::provider_icon(surface)");
    assert_source_contains(
        "gpui header",
        GPUI_HEADER,
        "Self::provider_selector(surface)",
    );
}

#[test]
fn adapters_keep_provider_selector_out_of_composer() {
    assert_source_absent("floem composer", FLOEM_COMPOSER, "on_vendor_select");
    assert_source_absent("egui composer", EGUI_COMPOSER, "select_vendor");
    assert_source_absent("gpui composer", GPUI_COMPOSER, "active_vendor_label");
}

#[test]
fn adapters_keep_core_toolbar_actions_in_header() {
    for (name, source) in [
        ("floem toolbar", FLOEM_TOOLBAR),
        ("egui header", EGUI_VIEW),
        ("gpui header", GPUI_HEADER),
    ] {
        assert_source_contains(name, source, "chrome.new_chat");
        assert_source_contains(name, source, "chrome.history");
        assert_source_absent(name, source, "chrome.settings");
    }
}

#[test]
fn adapters_keep_context_usage_next_to_primary_action() {
    assert_source_contains("floem composer", FLOEM_COMPOSER, "Self::right");
    assert_source_contains(
        "floem composer",
        FLOEM_COMPOSER,
        "FloemComposerUsageView::render",
    );
    assert_source_contains("egui composer", EGUI_COMPOSER, "usage_rect");
    assert_source_contains("egui composer", EGUI_COMPOSER, "primary_rect.left()");
    assert_source_contains("gpui composer", GPUI_COMPOSER, "usage_meter(surface)");
    assert_source_contains(
        "gpui composer",
        GPUI_COMPOSER,
        "primary_action(&surface.composer)",
    );
}

#[test]
fn standard_ui_does_not_mount_debug_or_settings_surfaces() {
    assert_source_absent("floem root", FLOEM_ROOT, "root_with_output_handoff");
    assert_source_absent("floem root", FLOEM_ROOT, "FloemSettingsSurface");
    assert_source_absent("floem root", FLOEM_ROOT, "output_handoff");
    assert_source_absent("floem root", FLOEM_ROOT, "debug");
}

#[test]
fn composer_submit_contract_is_explicitly_regression_tested() {
    assert_source_contains(
        "floem editor tests",
        FLOEM_COMPOSER_EDITOR_TESTS,
        "keypress_command_enter_submits_even_before_editor_update_event",
    );
    assert_source_contains(
        "floem editor tests",
        FLOEM_COMPOSER_EDITOR_TESTS,
        "keypress_enter_without_command_does_not_submit",
    );
}

#[test]
fn provider_selector_is_header_popup_with_visible_dropdown_hint() {
    assert_source_contains("floem selector", FLOEM_SELECTOR, "popout_menu");
    assert_source_contains("floem selector", FLOEM_SELECTOR, "CHEVRON_ICON");
    assert_source_contains(
        "floem selector",
        FLOEM_SELECTOR,
        "data-kcu-icon=\"provider-chevron\"",
    );
    assert_source_absent("floem composer", FLOEM_COMPOSER, "provider-chevron");
}

#[test]
fn resize_contract_keeps_thread_flexible_and_composer_bounded() {
    assert_source_contains("floem root", FLOEM_ROOT, ".size_full()");
    assert_source_contains("floem thread", FLOEM_THREAD, ".min_height(0.0)");
    assert_source_contains("floem thread", FLOEM_THREAD, ".flex_grow(1.0)");
    assert_source_contains("floem thread", FLOEM_THREAD, ".flex_shrink(1.0)");
    assert_source_contains(
        "floem composer view",
        FLOEM_COMPOSER_VIEW,
        "styles::CHAT_BODY_MAX_WIDTH",
    );
    assert_source_contains(
        "floem composer view",
        FLOEM_COMPOSER_VIEW,
        ".flex_shrink(0.0)",
    );
    assert_source_contains("floem thread", FLOEM_THREAD, "styles::CHAT_BODY_MAX_WIDTH");
    assert_source_contains("floem thread", FLOEM_THREAD, "AGENT_BUBBLE_WIDTH_PERCENT");
    assert_source_contains("floem thread", FLOEM_THREAD, "USER_BUBBLE_MAX_WIDTH");
}

#[test]
fn file_attach_intent_is_exercised_without_native_dialog_dependency() {
    assert_source_contains(
        "host e2e app",
        HOST_E2E_APP,
        "ChatInputDraft::request_path_drop",
    );
    assert_source_contains("host e2e app", HOST_E2E_APP, "add_attachment");
    assert_source_contains("host e2e app", HOST_E2E_APP, "attachment:count:");
    assert_source_contains("host e2e app", HOST_E2E_APP, "attachment:path_drop:");
}

#[test]
fn screenshot_request_requires_exact_visual_baseline() -> Result<(), serde_json::Error> {
    for source in screenshot_request_sources() {
        let request: serde_json::Value = serde_json::from_str(source)?;
        let hash = request
            .get("baseline_sha256")
            .and_then(serde_json::Value::as_str);

        assert!(matches!(hash, Some(value) if value.len() == 64));
        assert!(matches!(hash, Some(value) if value.chars().all(|it| it.is_ascii_hexdigit())));
    }
    Ok(())
}

#[test]
fn screenshot_matrix_covers_required_visual_states() -> Result<(), serde_json::Error> {
    let names = screenshot_request_sources()
        .into_iter()
        .map(screenshot_name)
        .collect::<Result<Vec<_>, _>>()?;

    for expected in [
        "empty-chat",
        "standard-chat",
        "thinking-chat",
        "narrow-chat",
        "tall-chat",
    ] {
        assert!(names.contains(&expected.to_string()));
    }
    Ok(())
}

fn screenshot_request_sources() -> Vec<&'static str> {
    vec![
        SCREENSHOT_REQUEST,
        SCREENSHOT_EMPTY_REQUEST,
        SCREENSHOT_THINKING_REQUEST,
        SCREENSHOT_NARROW_REQUEST,
        SCREENSHOT_TALL_REQUEST,
    ]
}

fn screenshot_name(source: &str) -> Result<String, serde_json::Error> {
    let request: serde_json::Value = serde_json::from_str(source)?;
    Ok(request["name"].as_str().unwrap_or("").to_string())
}

fn assert_source_contains(name: &str, source: &str, needle: &str) {
    assert!(
        source.contains(needle),
        "{name} must contain `{needle}` for adapter conformance"
    );
}

fn assert_source_absent(name: &str, source: &str, needle: &str) {
    assert!(
        !source.contains(needle),
        "{name} must not contain `{needle}` for adapter conformance"
    );
}
