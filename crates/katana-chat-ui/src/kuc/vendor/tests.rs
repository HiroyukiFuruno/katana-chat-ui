use super::{CONTROL_WIDTH, options, selector};
use crate::{
    ChatUiAction, ChatUiActionMapper, ChatUiVendorBarSurface, ChatUiVendorControlSurface,
    VendorOption,
};
use katana_ui_core::render_model::UiHostActionPlan;

const MODEL_CONTROL: &str = "chat.composer.control.0";

#[test]
fn model_option_uses_label_with_id_action_payload() -> Result<(), Box<dyn std::error::Error>> {
    let options = options(&vendor_bar(), MODEL_CONTROL, CONTROL_WIDTH)?;
    let option = options
        .first()
        .ok_or("open model dropdown must render an option")?;
    assert_eq!("Claude Sonnet 4.6", option.props().label);
    let actions = UiHostActionPlan::collect_from_root(option);
    let action = actions
        .iter()
        .find_map(selected_model_id)
        .ok_or("model option must have a typed action")?;
    assert_eq!("claude-sonnet-4-6", action);
    Ok(())
}

#[test]
fn provider_selector_keeps_the_human_label_and_pulldown_affordance()
-> Result<(), Box<dyn std::error::Error>> {
    let selector = selector(&vendor_bar(), None)?;

    assert_eq!("Claude Code ▾", selector.props().interaction.value);
    assert_eq!("Claude Code", selector.props().label);
    Ok(())
}

#[test]
fn unselected_provider_option_maps_to_typed_vendor_action() -> Result<(), Box<dyn std::error::Error>>
{
    let options = options(
        &unselected_vendor_bar(),
        "chat.chrome.provider-selector",
        CONTROL_WIDTH,
    )?;
    let option = options
        .get(1)
        .ok_or("unselected provider selector must render Codex")?;
    let actions = UiHostActionPlan::collect_from_root(option);
    let action = actions
        .iter()
        .find_map(selected_vendor_id)
        .ok_or("provider option must have a typed action")?;

    assert_eq!(action, "codex-cli");
    Ok(())
}

fn vendor_bar() -> ChatUiVendorBarSurface {
    ChatUiVendorBarSurface {
        active_vendor_id: "claude-code".to_string(),
        active_vendor_label: "Claude Code".to_string(),
        vendor_options: Vec::new(),
        vendor_selector_label: "Provider".to_string(),
        controls: vec![ChatUiVendorControlSurface {
            key: "model".to_string(),
            label: "Model".to_string(),
            value: "Claude Sonnet 4.6".to_string(),
            options: vec!["claude-sonnet-4-6".to_string()],
            option_labels: vec!["Claude Sonnet 4.6".to_string()],
            enabled: true,
        }],
        tools_visible: false,
        web_search_visible: false,
        permission_request: None,
    }
}

fn unselected_vendor_bar() -> ChatUiVendorBarSurface {
    let mut bar = vendor_bar();
    bar.active_vendor_id.clear();
    bar.active_vendor_label = "未選択".to_string();
    bar.vendor_options = vec![
        VendorOption::new("claude-code", "Claude Code"),
        VendorOption::new("codex-cli", "Codex"),
    ];
    bar.controls.clear();
    bar
}

fn selected_model_id(plan: &UiHostActionPlan) -> Option<String> {
    match ChatUiActionMapper::from_host_action(plan) {
        Ok(ChatUiAction::SelectVendorControl { key, value }) if key == "model" => Some(value),
        _ => None,
    }
}

fn selected_vendor_id(plan: &UiHostActionPlan) -> Option<String> {
    match ChatUiActionMapper::from_host_action(plan) {
        Ok(ChatUiAction::SelectVendor { vendor_id }) => Some(vendor_id),
        _ => None,
    }
}
