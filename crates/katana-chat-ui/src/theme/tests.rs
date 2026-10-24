use super::{IconRegistry, SvgIcon, ThemeOverride, ThemeTokens};

#[test]
fn theme_override_merges_accent_without_losing_defaults() {
    let theme = ThemeTokens::default().merge(ThemeOverride::new().with_accent("brand-accent"));

    assert_eq!(theme.colors.accent.name, "brand-accent");
    assert_eq!(theme.colors.surface.name, "surface");
}

#[test]
fn icon_registry_returns_host_override_by_asset_id() {
    let mut registry = IconRegistry::default();
    registry.override_icon(SvgIcon::new("send", "<svg id=\"host-send\"/>"));

    let icon = registry.resolve("send");

    assert_eq!(icon.asset_id, "send");
    assert_eq!(icon.svg, "<svg id=\"host-send\"/>");
}

#[test]
fn default_ollama_icon_is_not_placeholder_plus() {
    let registry = IconRegistry::default();

    let icon = registry.resolve("provider:ollama");

    assert!(icon.svg.contains("provider-ollama"));
    assert!(!icon.svg.contains("M8 12h8"));
}

#[test]
fn default_send_icon_is_stable_up_arrow() {
    let registry = IconRegistry::default();

    let icon = registry.resolve("send");

    assert!(icon.svg.contains(r#"data-kcu-icon="send""#));
    assert!(icon.svg.contains("M12 19V5"));
    assert!(!icon.svg.contains(r#"data-kcu-icon="send-alt""#));
}

#[test]
fn agent_provider_icons_are_not_generic_code_or_missing_icons() {
    let registry = IconRegistry::default();

    for (asset_id, expected_marker) in [
        ("provider:ollama", "provider-ollama"),
        ("provider:claude-code", "provider-claude-code"),
        ("provider:codex-cli", "provider-codex-cli"),
        ("provider:github-copilot", "provider-github-copilot"),
        ("provider:opencode", "provider-opencode"),
    ] {
        let icon = registry.resolve(asset_id);

        assert!(icon.svg.contains(expected_marker));
        assert!(!icon.svg.contains(r#"data-kcu-icon="missing""#));
        assert!(
            !icon
                .svg
                .contains("provider-codex-cli\" viewBox=\"0 0 24 24\"><path d=\"M10.3")
        );
        assert!(!icon.svg.contains("M8 12h8"));
    }
}
