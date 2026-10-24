use super::super::{ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeInput};
use super::helpers::{VIEWPORT, surface};
use crate::ChatUiAction;

#[test]
fn disabled_submit_hit_is_consumed_without_dispatching() -> Result<(), Box<dyn std::error::Error>> {
    let mut surface = surface("draft");
    surface.composer.send_available = false;
    surface.composer.send_enabled = false;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let submit = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains("Submit"))
        .ok_or_else(|| std::io::Error::other("missing disabled submit action"))?;
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
        !effects
            .iter()
            .any(|effect| matches!(effect, ChatUiRuntimeEffect::Action(ChatUiAction::Submit)))
    );
    assert!(effects.is_empty());
    Ok(())
}
