use super::{
    OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorConnectionKind,
    VendorControlProvider, VendorFact, VendorFactRegistry, VendorUiCapabilities, VendorUiProfile,
    VendorUiState, VendorUiSurface,
};

mod snapshot;

#[test]
fn capability_surface_hides_missing_vendor_affordances() {
    let surface = VendorUiSurface::from_capabilities(&VendorUiCapabilities::default());

    assert!(!surface.model_selector_visible);
    assert!(!surface.thinking_selector_visible);
    assert!(!surface.permission_mode_selector_visible);
}

#[test]
fn capability_surface_exposes_vendor_specific_controls() {
    let capabilities = VendorUiCapabilities::default()
        .with_models(vec!["sonnet".to_string()])
        .with_thinking_levels(vec!["high".to_string()])
        .with_permission_modes(vec!["ask".to_string()])
        .with_web_search();

    let surface = VendorUiSurface::from_capabilities(&capabilities);

    assert!(surface.model_selector_visible);
    assert!(surface.thinking_selector_visible);
    assert!(surface.permission_mode_selector_visible);
    assert!(surface.web_search_visible);
}

#[test]
fn profile_surface_tracks_active_vendor_identity() {
    let profile = VendorUiProfile::new(
        "claude",
        "Claude",
        VendorUiCapabilities::default()
            .with_models(vec!["sonnet".to_string()])
            .with_permission_modes(vec!["ask".to_string()]),
    );

    let surface = VendorUiSurface::from_profile(&profile);

    assert_eq!(surface.active_vendor_id, "claude");
    assert_eq!(surface.active_vendor_label, "Claude");
    assert!(surface.model_selector_visible);
    assert!(surface.permission_mode_selector_visible);
}

#[test]
fn builtin_vendor_facts_validate_official_references() {
    let registry = VendorFactRegistry::builtin();

    assert!(registry.validate().is_ok());
    assert!(registry.get("ollama").is_some());
    assert!(registry.get("github-copilot").is_some());
}

#[test]
fn builtin_vendor_fact_registry_matches_snapshot() {
    let registry = VendorFactRegistry::builtin();

    assert_eq!(
        snapshot::VendorRegistrySnapshot::render(&registry),
        include_str!("vendor_fact_registry.snapshot").trim()
    );
}

#[test]
fn ollama_surface_follows_official_capability_facts() {
    let registry = VendorFactRegistry::builtin();
    let state = VendorUiState::for_vendor("ollama")
        .with_endpoint("http://localhost:11434")
        .with_models(vec!["llama3".to_string()], "llama3")
        .with_thinking(vec!["false".to_string(), "low".to_string()], "low");

    let surface = VendorUiSurface::from_registry(&registry, &state);

    assert!(surface.model_selector_visible);
    assert!(surface.thinking_selector_visible);
    assert!(!surface.mode_selector_visible);
    assert!(!surface.permission_mode_selector_visible);
    assert!(surface.tool_approval_visible);
    assert!(!surface.web_search_visible);
    assert!(surface.controls.usage_visible);
    assert!(!surface.controls.account_usage_visible);
}

#[test]
fn registry_exposes_controls_through_interface() {
    let registry = VendorFactRegistry::builtin();
    let state = VendorUiState::for_vendor("claude-code")
        .with_available_vendors(vec![
            "claude-code".to_string(),
            "codex-cli".to_string(),
            "github-copilot".to_string(),
            "opencode".to_string(),
        ])
        .with_models(vec!["claude-sonnet-4-6".to_string()], "claude-sonnet-4-6")
        .with_thinking(vec!["default".to_string(), "high".to_string()], "default")
        .with_permission_modes(vec!["default".to_string(), "auto".to_string()], "default");

    let controls = registry.vendor_controls(&state);

    assert_eq!(controls.active_vendor_id, "claude-code");
    assert!(
        controls
            .vendor_options
            .iter()
            .any(|it| it.id == "claude-code")
    );
    assert!(controls.model.is_some());
    assert!(controls.thinking.is_some());
    assert!(controls.permission.is_some());
    assert!(controls.endpoint.is_none());
}

#[test]
fn default_state_does_not_select_direct_local_llm() {
    let state = VendorUiState::default();

    assert!(state.active_vendor_id.is_empty());
    assert!(state.available_vendor_ids.is_empty());
    assert!(state.endpoint.is_none());
}

#[test]
fn registry_does_not_fabricate_unknown_vendor_options() {
    let registry = VendorFactRegistry::builtin();
    let state = VendorUiState::for_vendor("codex-cli")
        .with_available_vendors(vec!["codex-cli".to_string(), "unknown-agent".to_string()]);

    let controls = registry.vendor_controls(&state);

    assert!(
        controls
            .vendor_options
            .iter()
            .any(|it| it.id == "codex-cli")
    );
    assert!(
        !controls
            .vendor_options
            .iter()
            .any(|it| it.id == "unknown-agent")
    );
}

#[test]
fn supported_capability_requires_official_reference() {
    let registry = VendorFactRegistry::new(vec![VendorFact {
        vendor_id: "bad".to_string(),
        display_name: "Bad".to_string(),
        connection_kind: VendorConnectionKind::LocalDirect,
        references: vec![reference()],
        endpoint: VendorCapabilityFact::supported(Vec::new(), Vec::new(), "missing reference"),
        model: unavailable(),
        mode: unavailable(),
        thinking: unavailable(),
        permission: unavailable(),
        tools: unavailable(),
        web_search: unavailable(),
        usage: unavailable(),
        account_usage: unavailable(),
        attachment: unavailable(),
    }]);

    assert!(registry.validate().is_err());
}

fn reference() -> OfficialReference {
    OfficialReference::new("ref", "https://example.com", "2026-05-06", "test")
}

fn unavailable() -> VendorCapabilityFact {
    VendorCapabilityFact::unavailable(VendorCapabilityStatus::Unsupported, "test")
}
