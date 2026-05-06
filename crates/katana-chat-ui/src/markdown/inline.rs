use super::{InlineSegment, TextBlock};
use comrak::nodes::{Node, NodeValue};

pub(super) struct InlineCollector;

impl InlineCollector {
    pub(super) fn collect(node: Node<'_>) -> TextBlock {
        let mut segments = Vec::new();
        for child in node.children() {
            Self::push_node(child, &mut segments);
        }
        TextBlock { segments }
    }

    fn push_node(node: Node<'_>, segments: &mut Vec<InlineSegment>) {
        let value = node.data.borrow().value.clone();
        match value {
            NodeValue::Text(text) => segments.push(InlineSegment::Text(text.to_string())),
            NodeValue::Code(code) => segments.push(InlineSegment::InlineCode(code.literal)),
            NodeValue::HtmlInline(html) => segments.push(InlineSegment::Text(html)),
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                segments.push(InlineSegment::Text("\n".to_string()));
            }
            NodeValue::Emph => Self::push_wrapped(node, segments, InlineSegment::Emphasis),
            NodeValue::Strong => Self::push_wrapped(node, segments, InlineSegment::Strong),
            NodeValue::Strikethrough => {
                Self::push_wrapped(node, segments, InlineSegment::Strikethrough);
            }
            NodeValue::Link(link) => Self::push_link(node, segments, link.url),
            NodeValue::Image(link) => Self::push_image(node, segments, link.url),
            _ => Self::push_children(node, segments),
        }
    }

    fn push_link(node: Node<'_>, segments: &mut Vec<InlineSegment>, destination: String) {
        let text = Self::collect(node).plain_text();
        segments.push(InlineSegment::Link { text, destination });
    }

    fn push_image(node: Node<'_>, segments: &mut Vec<InlineSegment>, destination: String) {
        let alt = Self::collect(node).plain_text();
        segments.push(InlineSegment::Image { alt, destination });
    }

    fn push_wrapped(
        node: Node<'_>,
        segments: &mut Vec<InlineSegment>,
        wrap: fn(String) -> InlineSegment,
    ) {
        let text = Self::collect(node).plain_text();
        segments.push(wrap(text));
    }

    fn push_children(node: Node<'_>, segments: &mut Vec<InlineSegment>) {
        for child in node.children() {
            Self::push_node(child, segments);
        }
    }
}
