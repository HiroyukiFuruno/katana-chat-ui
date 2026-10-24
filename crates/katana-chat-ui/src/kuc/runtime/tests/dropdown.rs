use super::super::{ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeInput};
use super::helpers::{NARROW_VIEWPORT, VIEWPORT, dropdown_surface, find_node, hit};
use crate::ChatUiAction;
use katana_ui_core::render_model::UiDimension;

#[test]
fn provider_dropdown_renders_option_and_dispatches_typed_vendor_action()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = dropdown_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let opened = open(
        &mut runtime,
        &surface,
        VIEWPORT,
        "chat.chrome.provider-selector",
    )?;
    let option = action_hit(&opened, "chat.chrome.provider-option.1")?;
    assert!(find_node(opened.tree.root(), "chat.dropdown.menu").is_some());
    let effects = press(&mut runtime, &opened, option)?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::SelectVendor { vendor_id }) if vendor_id == "codex-cli"
    )));
    Ok(())
}

#[test]
fn composer_dropdown_option_over_composer_dispatches_its_typed_action()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = dropdown_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let opened = open(
        &mut runtime,
        &surface,
        NARROW_VIEWPORT,
        "chat.composer.control.0",
    )?;
    let option = action_hit(&opened, "chat.composer.control.0.option.3")?;
    let input_top = opened.composer.bounds.y as f32;
    let input_bottom = opened
        .composer
        .bounds
        .y
        .saturating_add_unsigned(opened.composer.bounds.height) as f32;
    assert_option_over_input(option.1, input_top, input_bottom)?;
    assert_menu_covers_option_hits(&opened, "chat.composer.control.0.option.")?;
    let effects = press(&mut runtime, &opened, option)?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::SelectVendorControl { key, value })
            if key == "model" && value == "claude-instant"
    )));
    Ok(())
}

#[test]
fn outside_and_composer_presses_close_an_open_dropdown() -> Result<(), Box<dyn std::error::Error>> {
    let surface = dropdown_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let opened = open(
        &mut runtime,
        &surface,
        VIEWPORT,
        "chat.chrome.provider-selector",
    )?;
    close_after_press(&mut runtime, &surface, &opened, (0.0, 200.0))?;
    let opened = open(
        &mut runtime,
        &surface,
        VIEWPORT,
        "chat.chrome.provider-selector",
    )?;
    close_after_press(&mut runtime, &surface, &opened, composer_point(&opened))
}

fn close_after_press(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
    frame: &super::super::ChatUiRuntimeFrame,
    point: (f32, f32),
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = press(runtime, frame, point)?;
    assert_dropdown_closed(runtime, surface, VIEWPORT)
}

fn composer_point(frame: &super::super::ChatUiRuntimeFrame) -> (f32, f32) {
    (
        frame.composer.bounds.x as f32 + 4.0,
        frame.composer.bounds.y as f32 + 4.0,
    )
}

fn assert_option_over_input(
    option_y: f32,
    input_top: f32,
    input_bottom: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    if option_y > input_top && option_y < input_bottom {
        return Ok(());
    }
    Err(format!(
        "dropdown option center {option_y} must overlap composer input {input_top}..{input_bottom}"
    )
    .into())
}

fn assert_menu_covers_option_hits(
    frame: &super::super::ChatUiRuntimeFrame,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let menu = find_node(frame.tree.root(), "chat.dropdown.menu")
        .ok_or_else(|| std::io::Error::other("missing dropdown menu"))?;
    let UiDimension::Px(menu_height) = menu.props().common.height else {
        return Err(std::io::Error::other("dropdown menu must have pixel height").into());
    };
    let mut hits = frame
        .action_hits
        .iter()
        .filter(|hit| hit.action.target.as_str().starts_with(prefix));
    let first = hits
        .next()
        .ok_or_else(|| std::io::Error::other("missing dropdown option hit"))?;
    let (top, bottom) = hits.fold(
        (first.rect.y, first.rect.y.saturating_add(first.rect.height)),
        |(top, bottom), hit| {
            (
                top.min(hit.rect.y),
                bottom.max(hit.rect.y.saturating_add(hit.rect.height)),
            )
        },
    );
    if bottom.saturating_sub(top) <= usize::from(menu_height) {
        return Ok(());
    }
    Err("dropdown menu height must cover every option hit".into())
}

fn open(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
    viewport: super::super::ChatUiRuntimeViewport,
    node_id: &str,
) -> Result<super::super::ChatUiRuntimeFrame, Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, viewport)?;
    let trigger = hit(&frame.node_hits, node_id)
        .ok_or_else(|| std::io::Error::other("missing dropdown trigger"))?;
    let _ = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerPress {
            x: trigger.rect.center_point().0,
            y: trigger.rect.center_point().1,
            extend_selection: false,
        },
    )?;
    runtime.render(surface, viewport).map_err(Into::into)
}

fn action_hit(
    frame: &super::super::ChatUiRuntimeFrame,
    target: &str,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    frame
        .action_hits
        .iter()
        .find(|value| value.action.target.as_str() == target)
        .map(katana_ui_core::raster_host::UiTreeHostActionHit::center_point)
        .ok_or_else(|| std::io::Error::other("missing dropdown option").into())
}

fn press(
    runtime: &mut ChatUiRuntime,
    frame: &super::super::ChatUiRuntimeFrame,
    point: (f32, f32),
) -> Result<Vec<ChatUiRuntimeEffect>, Box<dyn std::error::Error>> {
    runtime
        .dispatch(
            frame,
            ChatUiRuntimeInput::PointerPress {
                x: point.0,
                y: point.1,
                extend_selection: false,
            },
        )
        .map_err(Into::into)
}

fn assert_dropdown_closed(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
    viewport: super::super::ChatUiRuntimeViewport,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, viewport)?;
    if find_node(frame.tree.root(), "chat.dropdown.menu").is_some() {
        return Err(std::io::Error::other("dropdown must close").into());
    }
    Ok(())
}
