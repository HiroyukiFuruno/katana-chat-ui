use katana_ui_core::render_model::{UiNode, UiTextSpan};

pub(super) fn node_with_id<'a>(
    nodes: &'a [UiNode],
    node_id: &str,
) -> Result<&'a UiNode, Box<dyn std::error::Error>> {
    nodes
        .iter()
        .find_map(|node| find_node(node, node_id))
        .ok_or_else(|| format!("node not found: {node_id}").into())
}

fn find_node<'a>(node: &'a UiNode, node_id: &str) -> Option<&'a UiNode> {
    if node.id().as_str() == node_id {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_node(child, node_id))
}

pub(super) fn code_node(nodes: &[UiNode]) -> Result<&UiNode, Box<dyn std::error::Error>> {
    node_with_id(nodes, "chat.message.1.body.5")?
        .children()
        .first()
        .ok_or_else(|| "code block must contain its text node".into())
}

pub(super) fn span_with_text<'a>(
    spans: &'a [UiTextSpan],
    text: &str,
) -> Result<&'a UiTextSpan, Box<dyn std::error::Error>> {
    spans
        .iter()
        .find(|span| span.text == text)
        .ok_or_else(|| format!("span not found: {text}").into())
}

pub(super) fn first_span(node: &UiNode) -> Result<&UiTextSpan, Box<dyn std::error::Error>> {
    node.props()
        .text
        .spans
        .first()
        .ok_or_else(|| format!("text span not found: {}", node.id().as_str()).into())
}
