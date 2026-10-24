use super::{VendorControlProvider, VendorFactRegistry, VendorUiState};

#[test]
fn provider_selector_excludes_local_runtime_fact() {
    let registry = VendorFactRegistry::builtin();
    let state = VendorUiState::for_vendor("katanagent").with_available_vendors(vec![
        "ollama".to_string(),
        "katanagent".to_string(),
        "claude-code".to_string(),
        "codex-cli".to_string(),
    ]);

    let controls = registry.vendor_controls(&state);

    assert!(contains_vendor_option(&controls, "katanagent"));
    assert!(contains_vendor_option(&controls, "claude-code"));
    assert!(contains_vendor_option(&controls, "codex-cli"));
    assert!(!contains_vendor_option(&controls, "ollama"));
}

fn contains_vendor_option(controls: &crate::VendorControlRenderModel, vendor_id: &str) -> bool {
    controls.vendor_options.iter().any(|it| it.id == vendor_id)
}
