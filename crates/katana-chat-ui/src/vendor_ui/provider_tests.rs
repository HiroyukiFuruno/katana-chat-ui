use super::{VendorControlProvider, VendorFactRegistry, VendorUiState};

#[test]
fn provider_selector_excludes_local_runtime_fact() {
    let registry = VendorFactRegistry::builtin();
    let state = VendorUiState::for_vendor("claude-code").with_available_vendors(vec![
        "ollama".to_string(),
        "claude-code".to_string(),
        "codex-cli".to_string(),
    ]);

    let controls = registry.vendor_controls(&state);

    assert!(
        controls
            .vendor_options
            .iter()
            .any(|it| it.id == "claude-code")
    );
    assert!(
        controls
            .vendor_options
            .iter()
            .any(|it| it.id == "codex-cli")
    );
    assert!(!controls.vendor_options.iter().any(|it| it.id == "ollama"));
}
