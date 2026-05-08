use super::*;
use katana_chat_ui::{
    CodeBlock, HeadingBlock, InlineSegment, ListItem, MarkdownBlock, TableBlock, TextBlock,
    markdown::ListBlock,
};

#[test]
fn compact_render_accepts_empty_fallback() {
    let _view = CompactMarkdownView::render(Vec::new(), "短い質問".to_string());
}

#[test]
fn compact_render_accepts_each_supported_block() {
    let blocks = vec![
        MarkdownBlock::Heading(HeadingBlock {
            level: 2,
            text: TextBlock::from_text("見出し"),
        }),
        MarkdownBlock::Paragraph(text_block()),
        MarkdownBlock::BlockQuote(TextBlock::from_text("引用")),
        MarkdownBlock::CodeBlock(CodeBlock {
            language: None,
            code: "let value = 1;".to_string(),
        }),
        MarkdownBlock::List(ListBlock {
            kind: ListKind::Ordered { start: 3 },
            items: vec![ListItem {
                checked: Some(true),
                content: TextBlock::from_text("項目"),
            }],
        }),
        MarkdownBlock::Table(TableBlock {
            headers: vec![TextBlock::from_text("A"), TextBlock::from_text("B")],
            rows: vec![vec![TextBlock::from_text("1"), TextBlock::from_text("2")]],
        }),
    ];

    let _view = CompactMarkdownView::render(blocks, String::new());
}

#[test]
fn compact_inline_text_preserves_visible_content() {
    assert_eq!(
        inline_text(&text_block()),
        "plain strong em strike code link image target"
    );
}

#[test]
fn compact_list_marker_and_heading_size_are_stable() {
    assert_eq!(list_marker(&ListKind::Unordered, 0, None), "-");
    assert_eq!(list_marker(&ListKind::Ordered { start: 4 }, 2, None), "6.");
    assert_eq!(list_marker(&ListKind::Unordered, 0, Some(false)), "[ ]");
    assert_eq!(heading_size(1), HEADING_LEVEL_ONE_SIZE);
    assert_eq!(heading_size(9), HEADING_LEVEL_REST_SIZE);
}

#[test]
fn compact_row_text_joins_cells() {
    let cells = vec![TextBlock::from_text("left"), TextBlock::from_text("right")];

    assert_eq!(row_text(&cells), "left | right");
}

fn text_block() -> TextBlock {
    TextBlock {
        segments: vec![
            InlineSegment::Text("plain ".to_string()),
            InlineSegment::Strong("strong ".to_string()),
            InlineSegment::Emphasis("em ".to_string()),
            InlineSegment::Strikethrough("strike ".to_string()),
            InlineSegment::InlineCode("code ".to_string()),
            InlineSegment::Link {
                text: "link ".to_string(),
                destination: "https://example.com".to_string(),
            },
            InlineSegment::Image {
                alt: "image target".to_string(),
                destination: "file:///tmp/image.png".to_string(),
            },
        ],
    }
}
