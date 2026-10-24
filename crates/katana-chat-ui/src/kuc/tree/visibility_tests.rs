use super::ChatUiTreeBuilder;
use crate::{ChatSession, ChatUiSurface};
use katana_ui_core::render_model::UiNode;

#[test]
fn standard_tree_omits_optional_debug_and_modal_surfaces() -> Result<(), Box<dyn std::error::Error>>
{
    let session = ChatSession::new();
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let tree = ChatUiTreeBuilder::build(&surface)?;

    for id in [
        "chat.history",
        "chat.vendor.capabilities",
        "chat.output-handoff",
        "chat.permission",
    ] {
        assert!(find_node(tree.tree().root(), id).is_none());
    }
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
