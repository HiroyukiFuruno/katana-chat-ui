use super::helpers::{VIEWPORT, find_node};
use crate::kuc::ChatUiRuntime;
use crate::{AgentActivityKind, ChatAgentEvent, ChatSession, ChatUiSurface};

#[test]
fn body_arrival_replaces_activity_in_the_rendered_tree() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = session();
    let id = session.start_assistant_stream("")?;
    assert_activity(&session, id, true)?;
    session.append_assistant_chunk("")?;
    assert_activity(&session, id, true)?;
    session.apply_agent_event(ChatAgentEvent::Chunk {
        content: "回答本文".into(),
    })?;
    assert_activity(&session, id, false)?;
    session.apply_agent_event(ChatAgentEvent::Activity {
        kind: AgentActivityKind::Editing,
    })?;
    assert_activity(&session, id, false)
}

#[test]
fn a_stream_with_initial_body_never_shows_work_activity() -> Result<(), Box<dyn std::error::Error>>
{
    let mut session = session();
    let id = session.start_assistant_stream("回答本文")?;
    assert_activity(&session, id, false)
}

#[test]
fn an_empty_failed_stream_clears_work_activity() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = session();
    let id = session.start_assistant_stream(" \n")?;
    assert_activity(&session, id, true)?;
    session.apply_agent_event(ChatAgentEvent::Failed {
        reason: "transport stopped".into(),
    })?;
    assert_activity(&session, id, false)
}

fn session() -> ChatSession {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session
}

fn assert_activity(
    session: &ChatSession,
    id: u64,
    visible: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    assert_eq!(
        find_node(frame.tree.root(), &format!("chat.message.{id}.activity")).is_some(),
        visible
    );
    Ok(())
}
