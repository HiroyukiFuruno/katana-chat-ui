use super::*;

#[test]
fn vendor_options_preserve_provider_id_and_label() {
    let options = vec![VendorOption {
        id: "claude-code".to_string(),
        label: "Claude Code".to_string(),
    }];

    let choices = VendorControlParts::vendor_options(&options);

    assert_eq!(choices[0].id, "claude-code");
    assert_eq!(choices[0].to_string(), "Claude Code");
}

#[test]
fn control_choice_uses_value_for_display_and_callback() {
    let choice = VendorControlParts::control_choice("gemma4:e4b");

    assert_eq!(choice.value, "gemma4:e4b");
    assert_eq!(choice.to_string(), "gemma4:e4b");
}

#[test]
fn control_text_keeps_label_and_current_value_together() {
    let control = control("model", "Model", "llama3");

    assert_eq!(VendorControlParts::control_text(&control), "Model: llama3");
}

#[test]
fn header_controls_hide_endpoint_from_composer_control_row() {
    let controls = vec![
        control("endpoint", "Endpoint", "http://localhost:11434"),
        control("model", "Model", "llama3"),
        control("thinking", "Thinking", "false"),
    ];

    let header_controls = VendorControlParts::header_controls(&controls);

    assert_eq!(header_controls.len(), 2);
    assert!(header_controls.iter().all(|it| it.key != "endpoint"));
}

#[test]
fn disabled_text_uses_muted_color() {
    assert_eq!(text_color(false), styles::COLOR_MUTED);
    assert_eq!(text_color(true), styles::COLOR_TEXT);
}

fn control(key: &str, label: &str, value: &str) -> ChatUiVendorControlSurface {
    ChatUiVendorControlSurface {
        key: key.to_string(),
        label: label.to_string(),
        value: value.to_string(),
        options: vec![value.to_string()],
        enabled: true,
    }
}
