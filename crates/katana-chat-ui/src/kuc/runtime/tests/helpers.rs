use super::super::{
    ChatUiRuntime, ChatUiRuntimeClipboard, ChatUiRuntimeEffect, ChatUiRuntimeInput,
    ChatUiRuntimeViewport,
};
use crate::{
    ChatSession, ChatUiAction, ChatUiSurface, VendorUiOption, VendorUiPermissionRequest,
    VendorUiState,
};
use katana_ui_core::raster_host::UiTreeNodeHit;
use katana_ui_core::render_model::UiNode;

pub(super) const VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(640, 480, 1.0);
pub(super) const NARROW_VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(240, 201, 1.0);

pub(super) fn surface(text: &str) -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text(text);
    ChatUiSurface::from_render_model(&session.render_model())
}

pub(super) fn permission_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    let mut state = VendorUiState::for_vendor("claude-code");
    state.pending_permission = Some(VendorUiPermissionRequest {
        request_id: 7,
        tool_call_id: Some("tool-7".to_string()),
        title: Some("Run the provided command".to_string()),
        details: vec!["command: cat ./README.md".to_string()],
        options: vec![VendorUiOption {
            id: "allow".to_string(),
            label: "許可".to_string(),
        }],
    });
    session.set_vendor_ui_state(state);
    ChatUiSurface::from_render_model(&session.render_model())
}

pub(super) fn dropdown_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("hello");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("claude-code")
            .with_available_vendors(vec!["claude-code".to_string(), "codex-cli".to_string()])
            .with_models(
                vec![
                    "claude-sonnet".to_string(),
                    "claude-opus".to_string(),
                    "claude-haiku".to_string(),
                    "claude-instant".to_string(),
                ],
                "claude-sonnet",
            ),
    );
    ChatUiSurface::from_render_model(&session.render_model())
}

pub(super) fn responsive_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("hello");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("claude-code")
            .with_models(vec!["claude-sonnet".to_string()], "claude-sonnet")
            .with_thinking(vec!["low".to_string(), "high".to_string()], "low")
            .with_permission_modes(vec!["ask".to_string(), "allow".to_string()], "ask"),
    );
    ChatUiSurface::from_render_model(&session.render_model())
}

pub(super) fn conversation_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("short");
    session.submit_draft()?;
    session.start_assistant_stream("agent response")?;
    session.finish_assistant_message()?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

pub(super) fn activity_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.start_assistant_stream(String::new())?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

pub(super) fn busy_composer_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.draft_mut().set_text("keep this draft");
    session.submit_draft()?;
    session.start_assistant_stream("partial response")?;
    session.draft_mut().set_text("keep this draft");
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

pub(super) fn markdown_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.start_assistant_stream(
        "# Heading\n\n```rust\nlet x = 1;\n```\n\n- item\n\n| A | B |\n|---|---|\n| c | d |",
    )?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

pub(super) fn long_markdown_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let paragraphs = (0..72)
        .map(|index| {
            format!(
                "## Section {index}\n\nLong Markdown content keeps this section visible after the thread scrolls through the shared KUC renderer."
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.start_assistant_stream(format!(
        "# Long document\n\n{paragraphs}\n\n```rust\nfn final_block() {{}}\n```"
    ))?;
    session.finish_assistant_message()?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

pub(super) fn focused_runtime(
    surface: &ChatUiSurface,
) -> Result<ChatUiRuntime, Box<dyn std::error::Error>> {
    let mut runtime = ChatUiRuntime::light(surface)?;
    let _ = focus(&mut runtime, surface)?;
    Ok(runtime)
}

pub(super) fn focus(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
) -> Result<Vec<ChatUiRuntimeEffect>, Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, VIEWPORT)?;
    let bounds = frame.composer.bounds;
    let caret_x = bounds
        .x
        .saturating_add_unsigned(bounds.width.saturating_sub(1));
    runtime
        .dispatch(
            &frame,
            ChatUiRuntimeInput::PointerPress {
                x: caret_x as f32,
                y: bounds.y as f32 + 9.0,
                extend_selection: false,
            },
        )
        .map_err(Into::into)
}

pub(super) fn has_draft(effects: &[ChatUiRuntimeEffect], expected: &str) -> bool {
    effects.iter().any(
        |effect| matches!(effect, ChatUiRuntimeEffect::DraftChanged(value) if value == expected),
    )
}

pub(super) fn has_clipboard(effects: &[ChatUiRuntimeEffect], expected: &str) -> bool {
    effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Clipboard(ChatUiRuntimeClipboard::Write(value)) if value == expected
    ))
}

pub(super) fn submit_count(effects: &[ChatUiRuntimeEffect]) -> usize {
    effects
        .iter()
        .filter(|effect| matches!(effect, ChatUiRuntimeEffect::Action(ChatUiAction::Submit)))
        .count()
}

pub(super) fn find_node<'a>(node: &'a UiNode, id: &str) -> Option<&'a UiNode> {
    if node.id().as_str() == id {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_node(child, id))
}

pub(super) fn hit<'a>(hits: &'a [UiTreeNodeHit], id: &str) -> Option<&'a UiTreeNodeHit> {
    hits.iter().find(|hit| hit.node_id.as_str() == id)
}
