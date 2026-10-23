use super::super::{ChatUiRuntime, ChatUiRuntimeViewport};
use super::helpers::{
    VIEWPORT, activity_surface, conversation_surface, dropdown_surface, find_node, hit,
    markdown_surface, surface,
};
use katana_ui_core::raster_host::UiTreeNodeHit;
use katana_ui_core::render_model::UiNodeKind;

const NARROW_VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(120, 300, 1.0);

#[test]
fn composer_is_fixed_to_the_viewport_bottom() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    assert_eq!(320, frame.composer.bounds.y);
    assert_eq!(560, frame.composer.bounds.width);
    assert!(
        frame
            .composer
            .bounds
            .y
            .saturating_add_unsigned(frame.composer.bounds.height)
            <= 480
    );
    Ok(())
}

#[test]
fn narrow_viewport_keeps_the_composer_input_visible() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, NARROW_VIEWPORT)?;
    assert_eq!(140, frame.composer.bounds.y);
    assert_eq!(120, frame.composer.bounds.width);
    Ok(())
}

#[test]
fn context_ring_is_directly_left_of_send() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let context = hit(&frame.node_hits, "chat.composer.usage")
        .ok_or_else(|| std::io::Error::other("missing context ring"))?;
    let send = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains("Submit"))
        .ok_or_else(|| std::io::Error::other("missing send action"))?;
    assert_eq!(28, context.rect.height);
    assert!(action_hit(&frame, "Attach").is_err());
    assert_eq!(32, send.rect.width);
    assert!(context.rect.x + 28 <= send.rect.x);
    Ok(())
}

#[test]
fn header_and_composer_controls_have_separate_visible_bounds()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = dropdown_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let selector = node_hit(&frame, "chat.chrome.provider-selector")?;
    let title = node_hit(&frame, "chat.chrome.title")?;
    let first = node_hit(&frame, "chat.composer.control.0")?;
    let second = node_hit(&frame, "chat.composer.control.1")?;
    let new_chat = action_hit(&frame, "NewChat")?;
    let history = action_hit(&frame, "ToggleHistory")?;

    assert_eq!(132, selector.rect.width);
    assert!(selector.rect.x.saturating_add(selector.rect.width) <= title.rect.x);
    assert_eq!(132, first.rect.width);
    assert!(first.rect.x.saturating_add(first.rect.width + 8) <= second.rect.x);
    assert_eq!((32, 28), (new_chat.rect.width, new_chat.rect.height));
    assert_eq!((32, 28), (history.rect.width, history.rect.height));
    Ok(())
}

fn node_hit<'a>(
    frame: &'a super::super::ChatUiRuntimeFrame,
    id: &str,
) -> Result<&'a katana_ui_core::raster_host::UiTreeNodeHit, Box<dyn std::error::Error>> {
    hit(&frame.node_hits, id).ok_or_else(|| format!("missing node hit: {id}").into())
}

fn action_hit<'a>(
    frame: &'a super::super::ChatUiRuntimeFrame,
    action: &str,
) -> Result<&'a katana_ui_core::raster_host::UiTreeHostActionHit, Box<dyn std::error::Error>> {
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(action))
        .ok_or_else(|| format!("missing action hit: {action}").into())
}

#[test]
fn user_and_agent_cards_use_the_actual_width_contract() -> Result<(), Box<dyn std::error::Error>> {
    let surface = conversation_surface()?;
    for width in [120, 320, 560] {
        let mut runtime = ChatUiRuntime::light(&surface)?;
        let viewport = ChatUiRuntimeViewport::new(width, VIEWPORT.height, 1.0);
        let frame = runtime.render(&surface, viewport)?;
        let user = node_hit(&frame, "chat.message.1.body.0")?;
        let agent = node_hit(&frame, "chat.message.2.body.0")?;
        let composer_width = usize::try_from(frame.composer.bounds.width)?;

        assert_eq!(composer_width, agent.rect.width, "viewport {width}");
        assert_eq!(0, agent.rect.x, "viewport {width}");
        assert!(user.rect.x + user.rect.width <= composer_width);
        for body in [user, agent] {
            assert_pointer_boundary(body);
        }
        if width == 560 {
            assert!(user.rect.x > agent.rect.x);
            assert!(user.rect.width < agent.rect.width);
        }
    }
    Ok(())
}

fn assert_pointer_boundary(body: &UiTreeNodeHit) {
    let left = body.rect.x as f32;
    let right = (body.rect.x + body.rect.width) as f32;
    let middle_y = body.rect.y as f32 + body.rect.height as f32 / 2.0;
    assert!(body.contains_point(left + 0.5, middle_y));
    assert!(body.contains_point(right - 0.5, middle_y));
    assert!(!body.contains_point(left - 0.5, middle_y));
    assert!(!body.contains_point(right + 0.5, middle_y));
}

#[test]
fn activity_card_remains_smaller_than_the_agent_composer_width()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = activity_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let activity = hit(&frame.node_hits, "chat.message.1.status")
        .ok_or_else(|| std::io::Error::other("missing activity card"))?;
    let composer_width = usize::try_from(frame.composer.bounds.width).unwrap_or(usize::MAX);

    assert!(activity.rect.width + 16 < composer_width);
    assert!(activity.rect.width + 16 >= 64);
    Ok(())
}

#[test]
fn runtime_frame_preserves_structured_markdown_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let surface = markdown_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let heading = find_node(frame.tree.root(), "chat.message.1.body.0")
        .ok_or_else(|| std::io::Error::other("missing heading"))?;
    let code = find_node(frame.tree.root(), "chat.message.1.body.1")
        .ok_or_else(|| std::io::Error::other("missing code block"))?;
    let list = find_node(frame.tree.root(), "chat.message.1.body.2")
        .ok_or_else(|| std::io::Error::other("missing list"))?;
    let table = find_node(frame.tree.root(), "chat.message.1.body.3")
        .ok_or_else(|| std::io::Error::other("missing table"))?;

    assert_eq!("heading", heading.props().text.role);
    assert_eq!(UiNodeKind::Panel, code.kind());
    assert_eq!(UiNodeKind::List, list.kind());
    assert_eq!(UiNodeKind::Grid, table.kind());
    Ok(())
}

#[path = "layout/responsive.rs"]
mod responsive;

#[path = "layout/requirements.rs"]
mod requirements;

#[path = "layout/visual.rs"]
mod visual;

#[path = "layout/measured_messages.rs"]
mod measured_messages;
