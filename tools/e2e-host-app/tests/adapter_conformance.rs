const FLOEM_TOOLBAR: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/toolbar.rs");
const FLOEM_SELECTOR: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/provider_icon_selector.rs");
const FLOEM_COMPOSER: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/composer_controls.rs");
const FLOEM_STYLES: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/styles.rs");
const EGUI_VIEW: &str = include_str!("../../../crates/katana-chat-ui-egui/src/view.rs");
const EGUI_COMPOSER: &str = include_str!("../../../crates/katana-chat-ui-egui/src/composer.rs");
const GPUI_HEADER: &str = include_str!("../../../crates/katana-chat-ui-gpui/src/view/header.rs");
const GPUI_COMPOSER: &str =
    include_str!("../../../crates/katana-chat-ui-gpui/src/view/composer.rs");
const GPUI_STYLES: &str = include_str!("../../../crates/katana-chat-ui-gpui/src/view/styles.rs");
const FLOEM_COMPOSER_VIEW: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/composer.rs");
const FLOEM_ROOT: &str = include_str!("../../../crates/katana-chat-ui-floem/src/widget/root.rs");
const FLOEM_MARKDOWN: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/markdown.rs");
const FLOEM_THREAD: &str =
    include_str!("../../../crates/katana-chat-ui-floem/src/widget/thread.rs");
const EGUI_MESSAGE: &str = include_str!("../../../crates/katana-chat-ui-egui/src/message.rs");
const GPUI_THREAD: &str = include_str!("../../../crates/katana-chat-ui-gpui/src/view/thread.rs");
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
const MANUAL_FLOEM_HOST: &str = include_str!("../../../tools/manual-host-floem/src/main.rs");
const MANUAL_EGUI_HOST: &str = include_str!("../../../tools/manual-host-egui/src/app.rs");
const MANUAL_GPUI_HOST: &str = include_str!("../../../tools/manual-host-gpui/src/main.rs");

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
    assert_source_contains("gpui header", GPUI_HEADER, "vendor_options.len()");
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
    for (name, source) in manual_host_sources() {
        assert_source_absent(name, source, "output JSON");
        assert_source_absent(name, source, "manual harness");
        assert_source_absent(name, source, "Ollama local LLM");
        assert_source_absent(name, source, "Floem host 起動確認");
        assert_source_absent(name, source, "Floem host 応答");
    }
}

#[test]
fn manual_hosts_mount_standard_chat_view_instead_of_local_chat_ui() {
    assert_source_contains(
        "manual floem host",
        MANUAL_FLOEM_HOST,
        "FloemChatView::render",
    );
    assert_source_contains("manual egui host", MANUAL_EGUI_HOST, "EguiChatView::render");
    assert_source_contains("manual gpui host", MANUAL_GPUI_HOST, "GpuiChatView::new");
    for (name, source) in manual_host_sources() {
        assert_source_absent(name, source, "会話");
        assert_source_absent(name, source, "入力欄");
        assert_source_absent(name, source, "出力物");
    }
}

#[test]
fn adapters_share_standard_layout_source() {
    assert_source_contains("floem styles", FLOEM_STYLES, "ChatUiLayoutSpec::DEFAULT");
    assert_source_contains("egui view", EGUI_VIEW, "ChatUiLayoutSpec::DEFAULT");
    assert_source_contains("gpui styles", GPUI_STYLES, "ChatUiLayoutSpec::DEFAULT");
    assert_source_contains("floem thread", FLOEM_THREAD, "CHAT_BODY_MAX_WIDTH");
    assert_source_contains("egui view", EGUI_VIEW, "chat_body_max_width");
    assert_source_contains("gpui styles", GPUI_STYLES, "chat_body_max_width");
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
fn adapters_render_markdown_from_structured_blocks() {
    assert_source_contains("floem markdown", FLOEM_MARKDOWN, "MarkdownBlock::CodeBlock");
    assert_source_contains("floem markdown", FLOEM_MARKDOWN, "MarkdownBlock::Table");
    assert_source_contains("egui message", EGUI_MESSAGE, "MarkdownBlock::CodeBlock");
    assert_source_contains("egui message", EGUI_MESSAGE, "for row in &table.rows");
    assert_source_contains("egui message", EGUI_MESSAGE, "list_marker(&list.kind");
    assert_source_contains("gpui thread", GPUI_THREAD, "structured_body(message)");
    assert_source_contains("gpui thread", GPUI_THREAD, "MarkdownBlock::CodeBlock");
    assert_source_contains("gpui thread", GPUI_THREAD, "MarkdownBlock::Table");
}

#[test]
fn adapters_keep_thread_messages_from_shared_surface() {
    assert_source_contains(
        "floem thread",
        FLOEM_THREAD,
        "surface.get().message_list.messages",
    );
    assert_source_contains(
        "egui message",
        EGUI_MESSAGE,
        "surface.message_list.messages",
    );
    assert_source_contains("gpui thread", GPUI_THREAD, "surface.message_list.messages");
    assert_source_absent("floem thread", FLOEM_THREAD, "role_label");
    assert_source_absent("egui message", EGUI_MESSAGE, "role_label");
    assert_source_absent("gpui thread", GPUI_THREAD, "role_label");
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

fn manual_host_sources() -> Vec<(&'static str, &'static str)> {
    vec![
        ("manual floem host", MANUAL_FLOEM_HOST),
        ("manual egui host", MANUAL_EGUI_HOST),
        ("manual gpui host", MANUAL_GPUI_HOST),
    ]
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
