use super::footer;
use crate::{ChatSession, ChatUiAction, ChatUiActionMapper, ChatUiSurface};

use katana_ui_core::render_model::{UiHostActionPlan, UiNode};

#[test]
fn standard_footer_has_no_attach_action() -> Result<(), Box<dyn std::error::Error>> {
    let session = ChatSession::new();
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let node = footer(
        &surface.composer,
        &surface.vendor_bar,
        &surface.usage,
        None,
        560,
    )?;

    assert!(find_node(&node, "chat.composer.attach").is_none());
    assert!(
        UiHostActionPlan::collect_from_root(&node)
            .iter()
            .all(|plan| !matches!(
                ChatUiActionMapper::from_host_action(plan),
                Ok(ChatUiAction::Attach)
            ))
    );
    Ok(())
}

fn find_node<'a>(node: &'a UiNode, id: &str) -> Option<&'a UiNode> {
    if node.id().as_str() == id {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_node(child, id))
}
