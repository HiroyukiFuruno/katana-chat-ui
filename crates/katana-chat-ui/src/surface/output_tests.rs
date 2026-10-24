use super::ChatUiSurface;
use crate::{ChatOutputKind, ChatSession, ChatSessionError, DiffCandidateOutput};

#[test]
fn surface_keeps_output_outside_chat_body() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    let model = session_with_diff_output(&mut session)?;
    let surface = ChatUiSurface::from_render_model(&model);

    assert_eq!(surface.messages.len(), 1);
    assert!(surface.messages[0].outputs.is_empty());
    assert_eq!(surface.output_handoff.outputs.len(), 1);
    assert!(surface.output_handoff.json_available);
    assert_eq!(model.outputs.len(), 1);
    assert!(!surface.messages[0].body.contains("src/lib.rs"));
    Ok(())
}

fn session_with_diff_output(
    session: &mut ChatSession,
) -> Result<crate::ChatRenderModel, ChatSessionError> {
    session.set_provider_configured("Claude Code");
    let assistant_id = session.start_assistant_stream("差分を返します")?;
    session.finish_assistant_message()?;
    session.add_output(
        assistant_id,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "src/lib.rs",
            "before",
            "after",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
            "src/lib.rs を更新",
        )),
    )?;
    Ok(session.render_model())
}
