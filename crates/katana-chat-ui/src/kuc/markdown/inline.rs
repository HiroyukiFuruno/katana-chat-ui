use crate::{InlineSegment, TextBlock};
use katana_ui_core::render_model::{
    UiNode, UiNodeKind, UiTextProps, UiTextSpan, UiTextSpanStyle, UiTextWrapMode,
};

pub(super) fn text_node(
    text: &TextBlock,
    node_id: &str,
    role: &str,
    mut prefix: Vec<UiTextSpan>,
) -> UiNode {
    prefix.extend(inline_spans(&text.segments));
    UiNode::new(UiNodeKind::Text, text.plain_text())
        .stable_node_id(node_id)
        .text(text_props(role, prefix))
}

pub(super) fn code_text_node(code: &str, node_id: &str, language: &str) -> UiNode {
    UiNode::new(UiNodeKind::Text, code)
        .stable_node_id(node_id)
        .text(text_props(
            &format!("markdown-code-{language}"),
            vec![code_span(code)],
        ))
}

fn code_span(code: &str) -> UiTextSpan {
    styled_span(
        code,
        UiTextSpanStyle {
            monospace: true,
            ..UiTextSpanStyle::default()
        },
    )
}

fn text_props(role: &str, spans: Vec<UiTextSpan>) -> UiTextProps {
    UiTextProps {
        role: role.to_string(),
        wrap: UiTextWrapMode::Wrap,
        spans,
        ..UiTextProps::default()
    }
}

fn inline_spans(segments: &[InlineSegment]) -> Vec<UiTextSpan> {
    segments.iter().map(inline_span).collect()
}

fn inline_span(segment: &InlineSegment) -> UiTextSpan {
    match segment {
        InlineSegment::Text(text) => UiTextSpan::plain(text),
        InlineSegment::Emphasis(text) => styled_span(text, italic_style()),
        InlineSegment::Strong(text) => styled_span(text, bold_style()),
        InlineSegment::Strikethrough(text) => styled_span(text, strikethrough_style()),
        InlineSegment::InlineCode(text) => styled_span(text, inline_code_style()),
        InlineSegment::Link { text, destination } => linked_span(text, destination),
        InlineSegment::Image { alt, destination } => image_span(alt, destination),
    }
}

fn italic_style() -> UiTextSpanStyle {
    UiTextSpanStyle {
        italic: true,
        ..UiTextSpanStyle::default()
    }
}

fn bold_style() -> UiTextSpanStyle {
    UiTextSpanStyle {
        bold: true,
        ..UiTextSpanStyle::default()
    }
}

fn strikethrough_style() -> UiTextSpanStyle {
    UiTextSpanStyle {
        strikethrough: true,
        ..UiTextSpanStyle::default()
    }
}

fn inline_code_style() -> UiTextSpanStyle {
    UiTextSpanStyle {
        monospace: true,
        inline_code: true,
        ..UiTextSpanStyle::default()
    }
}

fn styled_span(text: &str, style: UiTextSpanStyle) -> UiTextSpan {
    UiTextSpan {
        text: text.to_string(),
        style,
        link_target: String::new(),
    }
}

fn linked_span(text: &str, destination: &str) -> UiTextSpan {
    UiTextSpan {
        text: text.to_string(),
        style: UiTextSpanStyle {
            underline: true,
            ..UiTextSpanStyle::default()
        },
        link_target: destination.to_string(),
    }
}

fn image_span(alt: &str, destination: &str) -> UiTextSpan {
    let text = if alt.is_empty() { destination } else { alt };
    linked_span(text, destination)
}
