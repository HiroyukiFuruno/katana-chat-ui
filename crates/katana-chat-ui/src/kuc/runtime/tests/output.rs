use super::super::{
    ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeFrame, ChatUiRuntimeInput,
    ChatUiRuntimeViewport,
};
use crate::{ChatOutputKind, ChatSession, ChatUiAction, ChatUiSurface, DiffCandidateOutput};
use katana_ui_core::raster_host::{UiTreeHostActionHit, UiTreeNodeHit};

const NARROW_VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(430, 900, 1.0);
const WIDE_VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(900, 900, 1.0);
const OUTPUT_PREFIX: &str = "chat.output-handoff.output.0";

#[test]
fn output_body_and_actions_have_non_overlapping_render_bounds()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = output_surface()?;
    for viewport in [NARROW_VIEWPORT, WIDE_VIEWPORT] {
        let mut runtime = ChatUiRuntime::light(&surface)?;
        let frame = runtime.render(&surface, viewport)?;
        assert_output_bounds(&frame, viewport.width)?;
    }
    Ok(())
}

#[test]
fn output_apply_hit_dispatches_the_typed_output_action() -> Result<(), Box<dyn std::error::Error>> {
    let surface = output_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, NARROW_VIEWPORT)?;
    let apply = action_hit(&frame, ".action.1")?;
    let (x, y) = apply.center_point();
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
        ChatUiRuntimeEffect::Action(ChatUiAction::Output {
            kind: crate::HostActionKind::ApplyDiff,
            ..
        })
    )));
    Ok(())
}

fn assert_output_bounds(
    frame: &ChatUiRuntimeFrame,
    viewport_width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = node_hit(frame, ".body")?;
    let preview = action_hit(frame, ".action.0")?;
    let apply = action_hit(frame, ".action.1")?;
    assert_within(body, viewport_width)?;
    assert_within_action(preview, viewport_width)?;
    assert_within_action(apply, viewport_width)?;
    assert!(bottom(body) <= preview.rect.y);
    assert!(preview.rect.y.saturating_add(preview.rect.height) <= apply.rect.y);
    Ok(())
}

fn output_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    let message = session.start_assistant_stream("result")?;
    session.finish_assistant_message()?;
    session.add_output(
        message,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/generated.md",
            "before\ntext",
            "after\ntext",
            "--- a/tmp/generated.md\n+++ b/tmp/generated.md\n@@ -1 +1 @@\n-before\n+after",
            "sample diff",
        )),
    )?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn node_hit<'a>(
    frame: &'a ChatUiRuntimeFrame,
    suffix: &str,
) -> Result<&'a UiTreeNodeHit, Box<dyn std::error::Error>> {
    let id = format!("{OUTPUT_PREFIX}{suffix}");
    frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == id.as_str())
        .ok_or_else(|| format!("missing output node hit: {id}").into())
}

fn action_hit<'a>(
    frame: &'a ChatUiRuntimeFrame,
    suffix: &str,
) -> Result<&'a UiTreeHostActionHit, Box<dyn std::error::Error>> {
    let id = format!("{OUTPUT_PREFIX}{suffix}");
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == id.as_str())
        .ok_or_else(|| format!("missing output action hit: {id}").into())
}

fn assert_within(
    hit: &UiTreeNodeHit,
    viewport_width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if hit.rect.x.saturating_add(hit.rect.width) <= usize::try_from(viewport_width)? {
        return Ok(());
    }
    Err(format!("output node exceeds viewport: {}", hit.node_id.as_str()).into())
}

fn assert_within_action(
    hit: &UiTreeHostActionHit,
    viewport_width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if hit.rect.x.saturating_add(hit.rect.width) <= usize::try_from(viewport_width)? {
        return Ok(());
    }
    Err(format!(
        "output action exceeds viewport: {}",
        hit.action.target.as_str()
    )
    .into())
}

fn bottom(hit: &UiTreeNodeHit) -> usize {
    hit.rect.y.saturating_add(hit.rect.height)
}
