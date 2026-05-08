use super::ChatUiSurface;
use crate::{ChatSession, VendorUiState};

#[test]
fn surface_exposes_vendor_specific_affordances() {
    let surface = agent_vendor_surface();

    assert_eq!(surface.vendor_ui.active_vendor_id, "claude-code");
    assert!(has_vendor_option(&surface, "codex-cli"));
    assert!(surface.vendor_controls.endpoint.is_none());
    assert!(surface.vendor_ui.model_selector_visible);
    assert!(surface.vendor_ui.thinking_selector_visible);
    assert!(surface.vendor_ui.permission_mode_selector_visible);
    assert_vendor_bar_contract(&surface);
    assert!(!surface.vendor_controls.usage_visible);
    assert!(!surface.vendor_controls.account_usage_visible);
}

fn agent_vendor_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("claude-code")
            .with_available_vendors(vec![
                "claude-code".to_string(),
                "codex-cli".to_string(),
                "github-copilot".to_string(),
                "opencode".to_string(),
            ])
            .with_models(vec!["claude-sonnet-4-6".to_string()], "claude-sonnet-4-6")
            .with_thinking(vec!["default".to_string(), "low".to_string()], "default")
            .with_permission_modes(vec!["default".to_string(), "auto".to_string()], "default"),
    );

    ChatUiSurface::from_render_model(&session.render_model())
}

fn has_vendor_option(surface: &ChatUiSurface, vendor_id: &str) -> bool {
    surface
        .vendor_bar
        .vendor_options
        .iter()
        .any(|it| it.id == vendor_id)
}

fn assert_vendor_bar_contract(surface: &ChatUiSurface) {
    assert_eq!(surface.vendor_bar.active_vendor_label, "Claude Code");
    assert_eq!(surface.vendor_bar.vendor_selector_label, "Provider");
    assert!(!has_vendor_control(surface, "endpoint"));
    assert!(has_model_control(surface));
    assert!(has_vendor_control(surface, "permission"));
}

fn has_vendor_control(surface: &ChatUiSurface, key: &str) -> bool {
    surface.vendor_bar.controls.iter().any(|it| it.key == key)
}

fn has_model_control(surface: &ChatUiSurface) -> bool {
    surface
        .vendor_bar
        .controls
        .iter()
        .any(|it| it.key == "model" && it.options == vec!["claude-sonnet-4-6".to_string()])
}
