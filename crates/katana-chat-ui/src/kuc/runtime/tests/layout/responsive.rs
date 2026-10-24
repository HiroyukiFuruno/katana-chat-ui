use super::super::super::{ChatUiRuntime, ChatUiRuntimeFrame, ChatUiRuntimeViewport};
use super::super::helpers::{hit, responsive_surface};
use katana_ui_core::raster_host::{UiTreeHostActionHit, UiTreeNodeHit};

const VIEWPORT_HEIGHT: u32 = 480;
const WIDTHS: &[u32] = &[120, 320, 560];

#[test]
fn responsive_header_keeps_both_session_actions_inside_the_viewport()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = responsive_surface();
    for width in [320, 430, 560] {
        let mut runtime = ChatUiRuntime::light(&surface)?;
        let frame = runtime.render(&surface, ChatUiRuntimeViewport::new(width, 900, 1.0))?;
        for id in ["chat.chrome.new-chat", "chat.chrome.history"] {
            assert_within(node_hit(&frame, id)?, width)?;
        }
        let new_chat = node_hit(&frame, "chat.chrome.new-chat")?;
        let history = node_hit(&frame, "chat.chrome.history")?;
        assert!(new_chat.rect.x + new_chat.rect.width <= history.rect.x);
    }
    Ok(())
}

#[test]
fn responsive_composer_keeps_controls_and_primary_actions_reachable()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = responsive_surface();
    for &width in WIDTHS {
        let mut runtime = ChatUiRuntime::light(&surface)?;
        let viewport = ChatUiRuntimeViewport::new(width, VIEWPORT_HEIGHT, 1.0);
        let frame = runtime.render(&surface, viewport)?;
        assert_controls(&frame, width)?;
        assert_primary_actions(&frame, width)?;
    }
    Ok(())
}

fn assert_controls(
    frame: &ChatUiRuntimeFrame,
    width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    for id in [
        "chat.composer.control.0",
        "chat.composer.control.1",
        "chat.composer.control.2",
    ] {
        assert_within(node_hit(frame, id)?, width)?;
    }
    Ok(())
}

fn assert_primary_actions(
    frame: &ChatUiRuntimeFrame,
    width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(action_hit(frame, "Attach").is_err());
    let usage = node_hit(frame, "chat.composer.usage")?;
    let send = action_hit(frame, "Submit")?;
    assert_within(usage, width)?;
    assert_within_action(send, width)?;
    assert!(usage.rect.x.saturating_add(usage.rect.width) <= send.rect.x);
    Ok(())
}

fn node_hit<'a>(
    frame: &'a ChatUiRuntimeFrame,
    id: &str,
) -> Result<&'a UiTreeNodeHit, Box<dyn std::error::Error>> {
    hit(&frame.node_hits, id).ok_or_else(|| format!("missing node hit: {id}").into())
}

fn action_hit<'a>(
    frame: &'a ChatUiRuntimeFrame,
    action: &str,
) -> Result<&'a UiTreeHostActionHit, Box<dyn std::error::Error>> {
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(action))
        .ok_or_else(|| format!("missing action hit: {action}").into())
}

fn assert_within(hit: &UiTreeNodeHit, width: u32) -> Result<(), Box<dyn std::error::Error>> {
    let right = hit.rect.x.saturating_add(hit.rect.width);
    if right > usize::try_from(width)? {
        return Err(format!("node extends beyond viewport: {}", hit.node_id.as_str()).into());
    }
    Ok(())
}

fn assert_within_action(
    hit: &UiTreeHostActionHit,
    width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let right = hit.rect.x.saturating_add(hit.rect.width);
    if right > usize::try_from(width)? {
        return Err(format!(
            "action extends beyond viewport: {}",
            hit.action.target.as_str()
        )
        .into());
    }
    Ok(())
}
