use super::super::super::{ChatUiRuntime, ChatUiRuntimeFrame};
use super::super::helpers::{VIEWPORT, find_node, surface};
use super::{action_hit, node_hit};
use crate::{AgentActivityKind, ChatAgentEvent, ChatSession, ChatUiSurface, ThinkingLog};
use katana_ui_core::raster_host::{Canvas, UiTreeRenderArea};

#[test]
fn context_usage_raster_is_a_partial_ring_left_of_send() -> Result<(), Box<dyn std::error::Error>> {
    let mut surface = surface("hello");
    surface.usage.context_percentage = 50;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let usage = node_hit(&frame, "chat.composer.usage")?;
    let center_x = usage.rect.x + usage.rect.width / 2;
    let center_y = usage.rect.y + usage.rect.height / 2;

    assert_eq!(
        pixel(&frame, usage.rect.x, usage.rect.y),
        pixel(&frame, center_x, center_y)
    );
    assert_ne!(
        pixel(&frame, center_x, usage.rect.y + 2),
        pixel(&frame, center_x, center_y)
    );
    assert_ne!(
        pixel(&frame, usage.rect.x + 2, center_y),
        pixel(&frame, usage.rect.x + usage.rect.width - 3, center_y)
    );
    assert!(usage.rect.x + usage.rect.width <= action_hit(&frame, "Submit")?.rect.x);
    Ok(())
}

#[test]
fn agent_response_width_is_independent_of_body_length() -> Result<(), Box<dyn std::error::Error>> {
    let surface = two_agent_response_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let short = node_hit(&frame, "chat.message.2.body.0")?;
    let long = node_hit(&frame, "chat.message.4.body.0")?;
    let composer_width = usize::try_from(frame.composer.bounds.width)?;

    assert_eq!(composer_width, short.rect.width);
    assert_eq!(short.rect.width, long.rect.width);
    Ok(())
}

#[test]
fn thinking_execution_and_processing_stay_outside_response_width()
-> Result<(), Box<dyn std::error::Error>> {
    assert_auxiliary_width(processing_surface()?, "chat.message.1.activity")?;
    assert_auxiliary_width(executing_surface()?, "chat.message.1.activity")?;
    assert_auxiliary_width(thinking_surface()?, "chat.message.1.thinking")
}

#[test]
fn expanded_thinking_entries_remain_fully_visible() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = configured_session();
    session.start_assistant_stream_with_thinking(
        "",
        ThinkingLog::running(
            "Thinking",
            vec!["Inspect context".into(), "Plan response".into()],
        ),
    )?;
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let entries = node_hit(&frame, "chat.message.1.thinking.entries")?;
    assert!(
        entries.rect.height >= 40,
        "two thinking lines must remain visible: {:?}",
        entries.rect
    );
    assert_node_glyphs_visible(&runtime, &frame, "chat.message.1.thinking.entries")
}

fn assert_node_glyphs_visible(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries = node_hit(frame, node_id)?;
    let node = find_node(frame.tree.root(), node_id).ok_or("missing thinking entries node")?;
    let mut expected = Canvas::new(640, 480, runtime.canvas_background);
    runtime.renderer.render(
        &mut expected,
        node,
        UiTreeRenderArea {
            x: entries.rect.x,
            y: entries.rect.y,
            width: entries.rect.width,
            height: entries.rect.height,
            scroll_y: 0.0,
        },
    );
    assert_glyph_pixels_visible(frame, &expected, runtime.canvas_background);
    Ok(())
}

fn assert_glyph_pixels_visible(frame: &ChatUiRuntimeFrame, expected: &Canvas, background: u32) {
    let mut glyph_pixels = 0;
    for (index, color) in expected.pixels().iter().enumerate() {
        if *color != background {
            glyph_pixels += 1;
            assert_eq!(
                *color, frame.rgba.pixels[index],
                "thinking glyph clipped at pixel {index}"
            );
        }
    }
    assert!(glyph_pixels > 0);
}

fn two_agent_response_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = configured_session();
    session.draft_mut().set_text("first request");
    session.submit_draft()?;
    session.start_assistant_stream("short reply")?;
    session.finish_assistant_message()?;
    session.draft_mut().set_text("second request");
    session.submit_draft()?;
    session
        .start_assistant_stream("a deliberately longer agent response for width verification")?;
    session.finish_assistant_message()?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn processing_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = configured_session();
    session.start_assistant_stream("")?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn executing_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = configured_session();
    session.start_assistant_stream("")?;
    session.apply_agent_event(ChatAgentEvent::Activity {
        kind: AgentActivityKind::Executing,
    })?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn thinking_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = configured_session();
    session.start_assistant_stream_with_thinking(
        "",
        ThinkingLog::running("Thinking", vec!["Inspecting context".to_string()]),
    )?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn configured_session() -> ChatSession {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session
}

fn assert_auxiliary_width(
    surface: ChatUiSurface,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let node = node_hit(&frame, node_id)?;
    let composer_width = usize::try_from(frame.composer.bounds.width)?;
    assert!(
        node.rect.width < composer_width,
        "auxiliary node {node_id} width {} must be narrower than composer {composer_width}",
        node.rect.width
    );
    Ok(())
}

fn pixel(frame: &super::super::super::ChatUiRuntimeFrame, x: usize, y: usize) -> u32 {
    frame.rgba.pixels[y * frame.rgba.width as usize + x]
}
