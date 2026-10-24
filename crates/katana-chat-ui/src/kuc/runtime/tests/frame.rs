use super::super::{ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeInput};
use super::helpers::{VIEWPORT, find_node, long_markdown_surface, surface};
use crate::ChatUiAction;

#[test]
fn runtime_produces_a_kuc_rgba_frame() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;

    assert_eq!(640, frame.rgba.width);
    assert_eq!(480, frame.rgba.height);
    assert!(!frame.rgba.pixels.is_empty());
    assert_eq!(
        "chat.composer.input",
        frame.composer.text_surface.layout_identity
    );
    Ok(())
}

#[test]
fn light_runtime_uses_the_kuc_light_background() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;

    assert_eq!(Some(&0xffffff), frame.rgba.pixels.first());
    Ok(())
}

#[test]
fn composer_uses_the_light_surface_palette() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let x = frame.composer.bounds.x + i32::try_from(frame.composer.bounds.width / 2)?;
    let y = frame.composer.bounds.y + i32::try_from(frame.composer.bounds.height - 8)?;
    let index = usize::try_from(y)?
        .saturating_mul(usize::try_from(frame.rgba.width)?)
        .saturating_add(usize::try_from(x)?);

    assert_eq!(Some(&0xf3f3f3), frame.rgba.pixels.get(index));
    Ok(())
}

#[test]
fn typed_action_hit_dispatches_submit() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let submit = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains("Submit"))
        .ok_or_else(|| std::io::Error::other("missing submit action hit"))?;
    let (x, y) = submit.center_point();
    let effects = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;

    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, ChatUiRuntimeEffect::Action(ChatUiAction::Submit)))
    );
    Ok(())
}

#[test]
fn dropdown_hit_updates_kuc_tree_interaction() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let selector = frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "chat.chrome.provider-selector")
        .ok_or_else(|| std::io::Error::other("missing provider selector hit"))?;
    let (x, y) = selector.rect.center_point();
    let _ = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;
    let open = runtime.render(&surface, VIEWPORT)?;
    let selector = find_node(open.tree.root(), "chat.chrome.provider-selector")
        .ok_or_else(|| std::io::Error::other("missing provider selector node"))?;

    assert!(selector.props().interaction.open);
    Ok(())
}

#[test]
fn scroll_outside_composer_updates_tree_offset() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let effects = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::Scroll {
            x: 0.0,
            y: 0.0,
            delta_x: 0,
            delta_y: 40,
        },
    )?;

    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, ChatUiRuntimeEffect::Repaint))
    );
    assert_eq!(40, runtime.tree_scroll_y);
    let after_scroll = runtime.render(&surface, VIEWPORT)?;
    assert_eq!(320, after_scroll.composer.bounds.y);
    Ok(())
}

#[test]
fn long_markdown_scrolls_the_rendered_kuc_thread() -> Result<(), Box<dyn std::error::Error>> {
    let surface = long_markdown_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let before = runtime.render(&surface, VIEWPORT)?;
    let thread = find_node(before.tree.root(), "chat.thread")
        .ok_or_else(|| std::io::Error::other("missing KUC thread"))?;
    assert!(thread.props().scroll_area.content_height > thread.props().scroll_area.viewport_height);

    let _ = runtime.dispatch(
        &before,
        ChatUiRuntimeInput::Scroll {
            x: 8.0,
            y: 80.0,
            delta_x: 0,
            delta_y: 160,
        },
    )?;
    let after = runtime.render(&surface, VIEWPORT)?;
    let thread = find_node(after.tree.root(), "chat.thread")
        .ok_or_else(|| std::io::Error::other("missing scrolled KUC thread"))?;

    assert_eq!(160, runtime.tree_scroll_y);
    assert_eq!(160, thread.props().scroll_area.offset_y);
    assert!(thread_pixel_differences(&before, &after)? > 128);
    Ok(())
}

fn thread_pixel_differences(
    before: &super::super::ChatUiRuntimeFrame,
    after: &super::super::ChatUiRuntimeFrame,
) -> Result<usize, Box<dyn std::error::Error>> {
    let width = before.rgba.width as usize;
    let bottom = usize::try_from(before.composer.bounds.y.max(0))?;
    Ok((40..bottom)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let index = y.saturating_mul(width).saturating_add(*x);
            before.rgba.pixels.get(index) != after.rgba.pixels.get(index)
        })
        .count())
}
