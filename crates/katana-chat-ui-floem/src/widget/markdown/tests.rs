use super::*;
use katana_chat_ui::{InlineSegment, ListItem};

#[test]
fn inline_text_preserves_visible_markdown_content_without_markup_labels() {
    let block = TextBlock {
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
                alt: "image".to_string(),
                destination: "https://example.com/image.png".to_string(),
            },
        ],
    };

    assert_eq!(
        inline_text(&block),
        "plain strong em strike code link image"
    );
}

#[test]
fn image_segment_uses_destination_when_alt_text_is_empty() {
    let segment = InlineSegment::Image {
        alt: String::new(),
        destination: "file:///tmp/sample.png".to_string(),
    };

    assert_eq!(segment_text(&segment), "file:///tmp/sample.png");
}

#[test]
fn list_marker_uses_task_state_before_list_kind() {
    assert_eq!(list_marker(&ListKind::Unordered, 0, Some(true)), "[x]");
    assert_eq!(
        list_marker(&ListKind::Ordered { start: 4 }, 2, Some(false)),
        "[ ]"
    );
}

#[test]
fn list_marker_keeps_ordered_start_value() {
    assert_eq!(list_marker(&ListKind::Ordered { start: 7 }, 2, None), "9.");
    assert_eq!(list_marker(&ListKind::Unordered, 2, None), "-");
}

#[test]
fn table_rows_join_cell_plain_text_with_separator() {
    let row = vec![
        TextBlock::from_text("Priority"),
        TextBlock::from_text("Scope"),
        TextBlock::from_text("Version"),
    ];

    assert_eq!(row_text(&row), "Priority | Scope | Version");
}

#[test]
fn heading_size_is_bounded_to_three_display_sizes() {
    assert_eq!(heading_size(1), HEADING_LEVEL_ONE_SIZE);
    assert_eq!(heading_size(2), HEADING_LEVEL_TWO_SIZE);
    assert_eq!(heading_size(3), HEADING_LEVEL_REST_SIZE);
    assert_eq!(heading_size(6), HEADING_LEVEL_REST_SIZE);
}

#[test]
fn list_block_plain_text_uses_marker_and_item_content() {
    let items = [ListItem {
        checked: None,
        content: TextBlock::from_text("first"),
    }];

    assert_eq!(
        row_text(&[TextBlock::from_text(format!(
            "{} {}",
            list_marker(&ListKind::Unordered, 0, None),
            inline_text(&items[0].content)
        ))]),
        "- first"
    );
}
