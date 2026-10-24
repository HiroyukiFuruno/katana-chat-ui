use super::super::{ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeFrame, ChatUiRuntimeInput};
use super::helpers::{VIEWPORT, find_node, focus, has_clipboard, permission_surface, surface};
use crate::ChatUiAction;
use katana_ui_core::raster_host::UiTreeRenderArea;

#[test]
fn pointer_drag_outside_composer_keeps_kuc_selection_anchor()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let _ = focus(&mut runtime, &surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let bounds = frame.composer.bounds;
    let _ = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerDrag {
            x: -32.0,
            y: bounds.y as f32 + 9.0,
        },
    )?;
    let _ = runtime.dispatch(&frame, ChatUiRuntimeInput::PointerRelease)?;
    let copied = runtime.dispatch(&frame, ChatUiRuntimeInput::ClipboardCopy)?;

    assert!(has_clipboard(&copied, "hello"));
    Ok(())
}

#[test]
fn permission_modal_routes_only_its_topmost_action() -> Result<(), Box<dyn std::error::Error>> {
    let surface = permission_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let permission = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == "chat.permission.option.0")
        .ok_or_else(|| std::io::Error::other("missing permission action"))?;
    let (x, y) = permission.center_point();
    let effects = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;

    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::RespondPermission {
            request_id: 7,
            option_id: Some(option_id),
        }) if option_id == "allow"
    )));
    Ok(())
}

#[test]
fn permission_modal_blocks_background_composer_actions() -> Result<(), Box<dyn std::error::Error>> {
    let surface = permission_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let submit = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == "chat.composer.submit")
        .ok_or_else(|| std::io::Error::other("missing submit action"))?;
    let (x, y) = submit.center_point();
    let effects = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;

    assert!(effects.is_empty());
    Ok(())
}

#[test]
fn permission_modal_blocks_background_composer_scrolling() -> Result<(), Box<dyn std::error::Error>>
{
    let draft = (0..32)
        .map(|line| format!("line {line:02}: hidden draft content"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut surface = permission_surface();
    surface.composer.text = draft;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let before = runtime.render(&surface, VIEWPORT)?;
    let bounds = before.composer.bounds;
    let _ = runtime.dispatch(
        &before,
        ChatUiRuntimeInput::Scroll {
            x: bounds.x as f32 + 8.0,
            y: bounds.y as f32 + 8.0,
            delta_x: 0,
            delta_y: 48,
        },
    )?;
    let after = runtime.render(&surface, VIEWPORT)?;
    assert_eq!(
        before.composer.text_surface.viewport.scroll_y,
        after.composer.text_surface.viewport.scroll_y
    );
    assert_eq!(0, runtime.tree_scroll_y);
    Ok(())
}

#[test]
fn permission_frame_shows_operation_and_keeps_long_details_scrollable()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = long_permission_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let before = runtime.render(&surface, VIEWPORT)?;
    assert_long_context_tree(&before)?;
    let after = scroll_permission_context(&mut runtime, &surface, &before)?;
    assert_permission_allow_routes(&mut runtime, &after)
}

fn long_permission_surface() -> Result<crate::ChatUiSurface, Box<dyn std::error::Error>> {
    let mut surface = permission_surface();
    let request = surface
        .vendor_bar
        .permission_request
        .as_mut()
        .ok_or_else(|| std::io::Error::other("missing permission request"))?;
    request.details = vec![format!(
        "command: {}",
        "cat /workspace/private.txt && ".repeat(80)
    )];
    request.title = Some("Run the provided command".to_string());
    request.tool_call_id = Some("tool-long".to_string());
    request
        .details
        .push("literal: <script>must remain plain text</script>".to_string());
    Ok(surface)
}

fn assert_long_context_tree(frame: &ChatUiRuntimeFrame) -> Result<(), Box<dyn std::error::Error>> {
    let operation = find_node(frame.tree.root(), "chat.permission.context.detail.0")
        .ok_or_else(|| std::io::Error::other("missing operation detail in KUC tree"))?;
    assert!(operation.props().label.contains("/workspace/private.txt"));
    let literal = find_node(frame.tree.root(), "chat.permission.context.detail.1")
        .ok_or_else(|| std::io::Error::other("missing literal operation text"))?;
    assert_eq!(
        literal.props().label,
        "literal: <script>must remain plain text</script>"
    );
    let detail_area = find_node(frame.tree.root(), "chat.permission.details-scroll")
        .ok_or_else(|| std::io::Error::other("missing permission detail viewport"))?;
    assert!(
        detail_area.props().scroll_area.content_height
            > detail_area.props().scroll_area.viewport_height
    );
    assert!(
        frame
            .action_hits
            .iter()
            .any(|hit| hit.action.target.as_str() == "chat.permission.option.0")
    );
    Ok(())
}

fn scroll_permission_context(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
    frame: &ChatUiRuntimeFrame,
) -> Result<ChatUiRuntimeFrame, Box<dyn std::error::Error>> {
    assert_backdrop_wheel_is_consumed(runtime, frame)?;
    let (x, y) = visible_permission_title_center(runtime, frame)?;
    let _ = runtime.dispatch(
        frame,
        ChatUiRuntimeInput::Scroll {
            x,
            y,
            delta_x: 0,
            delta_y: 48,
        },
    )?;
    let after = runtime.render(surface, VIEWPORT)?;
    assert_permission_context_scrolled(frame, &after)?;
    Ok(after)
}

fn assert_backdrop_wheel_is_consumed(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = runtime.dispatch(
        frame,
        ChatUiRuntimeInput::Scroll {
            x: 4.0,
            y: 4.0,
            delta_x: 0,
            delta_y: 48,
        },
    )?;
    assert_eq!(
        runtime.tree_scroll_y, 0,
        "modal backdrop wheel must not scroll the thread"
    );
    assert_eq!(
        runtime.permission_scroll_y, 0,
        "modal backdrop wheel must not scroll hidden details"
    );
    Ok(())
}

fn visible_permission_title_center(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let document_hits = runtime.renderer.document_node_hit_rects(
        frame.tree.root(),
        UiTreeRenderArea {
            x: 0,
            y: 0,
            width: VIEWPORT.width as usize,
            height: VIEWPORT.height as usize,
            scroll_y: 0.0,
        },
    );
    let area_hit = document_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "chat.permission.title")
        .ok_or_else(|| std::io::Error::other("missing visible permission context hit area"))?;
    Ok(area_hit.rect.center_point())
}

fn assert_permission_context_scrolled(
    before: &ChatUiRuntimeFrame,
    after: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let detail_area = find_node(after.tree.root(), "chat.permission.details-scroll")
        .ok_or_else(|| std::io::Error::other("missing scrolled permission viewport"))?;
    assert!(detail_area.props().scroll_area.offset_y > 0);
    assert_ne!(
        before.rgba.pixels, after.rgba.pixels,
        "scroll must change the rendered frame"
    );
    Ok(())
}

fn assert_permission_allow_routes(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let approval = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == "chat.permission.option.0")
        .ok_or_else(|| std::io::Error::other("approval action was clipped"))?;
    let (x, y) = approval.center_point();
    let effects = runtime.dispatch(
        frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::RespondPermission {
            request_id: 7,
            option_id: Some(option_id),
        }) if option_id == "allow"
    )));
    Ok(())
}

#[test]
fn four_permission_options_and_cancel_remain_inside_the_viewport()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = four_option_permission_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    for index in 0..4 {
        assert_action_inside_viewport(&frame, &format!("chat.permission.option.{index}"))?;
    }
    assert_action_inside_viewport(&frame, "chat.permission.cancel")
}

fn four_option_permission_surface() -> Result<crate::ChatUiSurface, Box<dyn std::error::Error>> {
    let mut surface = permission_surface();
    let request = surface
        .vendor_bar
        .permission_request
        .as_mut()
        .ok_or_else(|| std::io::Error::other("missing permission request"))?;
    let template = request
        .options
        .first()
        .cloned()
        .ok_or_else(|| std::io::Error::other("missing permission option template"))?;
    request.options = (0..4)
        .map(|index| {
            let mut option = template.clone();
            option.id = format!("option-{index}");
            option.label = format!("選択肢 {index} を許可");
            option
        })
        .collect();
    Ok(surface)
}

fn assert_action_inside_viewport(
    frame: &ChatUiRuntimeFrame,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let hit = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == target)
        .ok_or_else(|| {
            std::io::Error::other(format!("missing visible permission action {target}"))
        })?;
    assert!(hit.rect.y.saturating_add(hit.rect.height) <= VIEWPORT.height as usize);
    Ok(())
}

#[test]
fn permission_frame_labels_missing_operation_details() -> Result<(), Box<dyn std::error::Error>> {
    let mut surface = permission_surface();
    let request = surface
        .vendor_bar
        .permission_request
        .as_mut()
        .ok_or_else(|| std::io::Error::other("missing permission request"))?;
    request.details.clear();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let unavailable = find_node(frame.tree.root(), "chat.permission.details-unavailable")
        .ok_or_else(|| std::io::Error::other("missing unavailable operation warning"))?;
    assert!(unavailable.props().label.contains("提供されていません"));
    Ok(())
}

#[test]
fn long_permission_title_is_included_in_measured_context_height()
-> Result<(), Box<dyn std::error::Error>> {
    let mut surface = permission_surface();
    let request = surface
        .vendor_bar
        .permission_request
        .as_mut()
        .ok_or_else(|| std::io::Error::other("missing permission request"))?;
    request.details.clear();
    request.title = Some("Provided operation title ".repeat(40));
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let details = find_node(frame.tree.root(), "chat.permission.details-scroll")
        .ok_or_else(|| std::io::Error::other("missing permission detail viewport"))?;
    assert!(
        details.props().scroll_area.content_height > details.props().scroll_area.viewport_height
    );
    Ok(())
}
