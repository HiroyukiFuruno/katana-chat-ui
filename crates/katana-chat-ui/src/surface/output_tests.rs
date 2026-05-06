use super::ChatUiSurface;
use crate::{
    ChatOutputKind, ChatSession, ChatSessionError, ChatUiConfig, ChatUiOptions, DiffCandidateOutput,
};

#[test]
fn surface_keeps_output_outside_chat_body() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    let model = session_with_diff_output(&mut session)?;
    let surface = ChatUiSurface::from_render_model(&model);

    assert_eq!(surface.messages.len(), 1);
    assert_eq!(surface.output_handoff.outputs.len(), 1);
    assert!(surface.output_handoff.json_available);
    assert!(!surface.debug.enabled);
    assert_eq!(model.outputs.len(), 1);
    assert!(!surface.messages[0].body.contains("src/lib.rs"));
    Ok(())
}

#[test]
fn surface_exposes_output_text_only_when_debug_option_is_enabled() -> Result<(), ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_debug_enabled(true);
    session_with_diff_output(&mut session)?;

    let surface = ChatUiSurface::from_render_model(&session.render_model());

    assert!(surface.debug.enabled);
    assert_eq!(surface.debug.label, "Output");
    assert!(surface.debug.output_handoff_text.contains("DiffCandidate"));
    assert!(!surface.messages[0].body.contains("output JSON"));
    Ok(())
}

#[test]
fn config_options_drive_debug_surface() -> Result<(), ChatSessionError> {
    let config = ChatUiConfig::default().with_options(ChatUiOptions::default().with_debug(true));
    let mut session = ChatSession::with_config(config);
    session_with_diff_output(&mut session)?;

    let surface = ChatUiSurface::from_render_model(&session.render_model());

    assert!(session.render_model().ui_options.debug);
    assert!(surface.debug.enabled);
    assert!(surface.debug.output_handoff_text.contains("DiffCandidate"));
    Ok(())
}

fn session_with_diff_output(
    session: &mut ChatSession,
) -> Result<crate::ChatRenderModel, ChatSessionError> {
    session.set_provider_configured("ollama");
    let assistant_id = session.start_assistant_stream("差分を返します")?;
    session.finish_assistant_message()?;
    session.add_output(
        assistant_id,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "src/lib.rs",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
        )),
    )?;
    Ok(session.render_model())
}
