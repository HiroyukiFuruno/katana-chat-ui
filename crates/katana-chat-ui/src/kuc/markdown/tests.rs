use super::markdown_nodes;
use crate::markdown::ListBlock;
use crate::{InlineSegment, ListItem, ListKind, MarkdownBlock, MarkdownSubset, TextBlock};

mod helpers;

use helpers::{code_node, first_span, node_with_id, span_with_text};

#[test]
fn markdown_nodes_keep_all_block_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = MarkdownSubset::parse(
        "## **Heading**\n\nparagraph\n\n> quote\n\n- item\n\n| head |\n| --- |\n| cell |\n\n```rust\nfn main() {}\n```",
    );
    let nodes = markdown_nodes(&parsed.blocks, "chat.message.1.body");
    let heading = node_with_id(&nodes, "chat.message.1.body.0")?;
    let quote = node_with_id(&nodes, "chat.message.1.body.2")?;
    let list = node_with_id(&nodes, "chat.message.1.body.3")?;
    let table = node_with_id(&nodes, "chat.message.1.body.4")?;

    assert_eq!(6, nodes.len());
    assert!(first_span(heading)?.style.bold);
    assert_eq!("heading-2", heading.props().text.role);
    assert_eq!(1, quote.children().len());
    assert_eq!(1, list.children().len());
    assert_eq!(2, table.children().len());
    assert!(first_span(code_node(&nodes)?)?.style.monospace);
    Ok(())
}

#[test]
fn markdown_nodes_keep_inline_styles_and_destinations() -> Result<(), Box<dyn std::error::Error>> {
    let blocks = vec![MarkdownBlock::Paragraph(TextBlock {
        segments: inline_fixture(),
    })];
    let nodes = markdown_nodes(&blocks, "chat.message.2.body");
    let spans = &node_with_id(&nodes, "chat.message.2.body.0")?
        .props()
        .text
        .spans;

    assert!(span_with_text(spans, "em")?.style.italic);
    assert!(span_with_text(spans, "strike")?.style.strikethrough);
    assert!(span_with_text(spans, "code")?.style.inline_code);
    assert!(span_with_text(spans, "code")?.style.monospace);
    assert_eq!(
        "https://example.com",
        span_with_text(spans, "link")?.link_target
    );
    assert_eq!(
        "https://example.com/image.png",
        span_with_text(spans, "image")?.link_target
    );
    Ok(())
}

#[test]
fn markdown_nodes_render_list_markers_and_readonly_tasks() -> Result<(), Box<dyn std::error::Error>>
{
    let blocks = vec![MarkdownBlock::List(ListBlock {
        kind: ListKind::Ordered { start: 3 },
        items: vec![ListItem {
            checked: Some(true),
            content: TextBlock::from_text("task"),
        }],
    })];
    let nodes = markdown_nodes(&blocks, "chat.message.3.body");
    let item = node_with_id(&nodes, "chat.message.3.body.0.item.0")?;
    let task = item
        .children()
        .iter()
        .find(|node| node.id().as_str().ends_with(".task"));
    let text = item
        .children()
        .iter()
        .find(|node| node.id().as_str().ends_with(".text"));

    let task = task.ok_or_else(|| "task list item must keep its checkbox".to_string())?;
    let text = text.ok_or_else(|| "task list item must keep its text".to_string())?;
    assert!(task.props().disabled);
    assert_eq!("3. ", first_span(text)?.text);
    Ok(())
}

#[test]
fn markdown_nodes_saturate_ordered_list_marker() -> Result<(), Box<dyn std::error::Error>> {
    let blocks = vec![MarkdownBlock::List(ListBlock {
        kind: ListKind::Ordered { start: u64::MAX },
        items: vec![
            ListItem {
                checked: None,
                content: TextBlock::from_text("first"),
            },
            ListItem {
                checked: None,
                content: TextBlock::from_text("second"),
            },
        ],
    })];
    let nodes = markdown_nodes(&blocks, "chat.message.overflow.body");

    assert_eq!(
        format!("{}. ", u64::MAX),
        first_span(node_with_id(
            &nodes,
            "chat.message.overflow.body.0.item.1.text"
        )?)?
        .text
    );
    Ok(())
}

#[test]
fn markdown_nodes_keep_raw_html_text_and_image_alt_link() -> Result<(), Box<dyn std::error::Error>>
{
    let parsed =
        MarkdownSubset::parse("<script>alert(1)</script>\n\n![alt](https://example.com/image.png)");
    let nodes = markdown_nodes(&parsed.blocks, "chat.message.4.body");
    let html = node_with_id(&nodes, "chat.message.4.body.0")?;
    let image = node_with_id(&nodes, "chat.message.4.body.1")?;

    assert_eq!("<script>alert(1)</script>\n", first_span(html)?.text);
    assert_eq!("alt", first_span(image)?.text);
    assert_eq!(
        "https://example.com/image.png",
        first_span(image)?.link_target
    );
    Ok(())
}

#[test]
fn mixed_bidi_code_spans_preserve_paragraph_separators() -> Result<(), Box<dyn std::error::Error>> {
    for separator in ["\r", "\n", "\r\n", "\u{2028}", "\u{2029}"] {
        let body = format!("שלום{separator}مرحبا");
        let code = format!("let x = 1;{separator}x");
        let blocks = vec![MarkdownBlock::Paragraph(TextBlock {
            segments: vec![
                InlineSegment::Text(body.clone()),
                InlineSegment::InlineCode(code.clone()),
                InlineSegment::Text(" world".to_string()),
            ],
        })];
        let nodes = markdown_nodes(&blocks, "chat.message.bidi.body");
        let text = &node_with_id(&nodes, "chat.message.bidi.body.0")?
            .props()
            .text;
        let rendered_text: String = text.spans.iter().map(|span| span.text.as_str()).collect();
        assert_eq!(format!("{body}{code} world"), rendered_text);
        let span = span_with_text(&text.spans, &code)?;
        assert!(span.style.monospace);
        assert!(span.style.inline_code);
    }
    Ok(())
}

fn inline_fixture() -> Vec<InlineSegment> {
    vec![
        InlineSegment::Emphasis("em".to_string()),
        InlineSegment::Strikethrough("strike".to_string()),
        InlineSegment::InlineCode("code".to_string()),
        InlineSegment::Link {
            text: "link".to_string(),
            destination: "https://example.com".to_string(),
        },
        InlineSegment::Image {
            alt: "image".to_string(),
            destination: "https://example.com/image.png".to_string(),
        },
    ]
}
