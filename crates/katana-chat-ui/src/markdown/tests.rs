use super::{InlineSegment, MarkdownBlock, MarkdownSubset};

#[test]
fn parser_keeps_gfm_markdown_as_structured_blocks() {
    let parsed = MarkdownSubset::parse(
        "# Title\n\nIntro **strong** *em* ~~gone~~ `code` <https://example.com>\n\n\
        ```rust\nfn main() {}\n```\n\n> quote\n\n- [x] one\n- [ ] two\n\n\
        | A | B |\n|---|---|\n| c | d |\n",
    );

    assert_gfm_blocks(&parsed);
    assert_gfm_inline_segments(&parsed);
    assert_gfm_task_items(&parsed);
}

#[test]
fn parser_treats_raw_html_as_text_not_executable_html() {
    let parsed = MarkdownSubset::parse("<script>alert(1)</script>\n\n<style>body{}</style>");

    assert!(parsed.plain_text().contains("<script>"));
    assert!(parsed.plain_text().contains("<style>"));
}

#[test]
fn parser_keeps_image_markdown_as_safe_inline_link() {
    let parsed = MarkdownSubset::parse("![alt text](https://example.com/image.png)");

    assert!(contains_inline_segment(&parsed, |it| {
        matches!(
        it,
        InlineSegment::Image {
            alt,
            destination
        } if alt == "alt text" && destination == "https://example.com/image.png"
        )
    }));
}

#[test]
fn parser_keeps_nested_code_fence_inside_longer_code_block() {
    let parsed = MarkdownSubset::parse("````markdown\n```rust\nfn main() {}\n```\n````\n");
    let code_blocks = parsed
        .blocks
        .iter()
        .filter_map(|block| match block {
            MarkdownBlock::CodeBlock(code) => Some(code),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(code_blocks.len(), 1);
    assert_eq!(code_blocks[0].language.as_deref(), Some("markdown"));
    assert!(code_blocks[0].code.contains("```rust"));
    assert!(code_blocks[0].code.contains("fn main() {}"));
}

fn contains_inline_segment(
    parsed: &MarkdownSubset,
    predicate: impl Fn(&InlineSegment) -> bool,
) -> bool {
    parsed.blocks.iter().any(|block| match block {
        MarkdownBlock::Heading(it) => text_has_segment(&it.text, &predicate),
        MarkdownBlock::Paragraph(it) | MarkdownBlock::BlockQuote(it) => {
            text_has_segment(it, &predicate)
        }
        MarkdownBlock::List(list) => list
            .items
            .iter()
            .any(|it| text_has_segment(&it.content, &predicate)),
        MarkdownBlock::Table(table) => table
            .headers
            .iter()
            .chain(table.rows.iter().flatten())
            .any(|it| text_has_segment(it, &predicate)),
        MarkdownBlock::CodeBlock(_) => false,
    })
}

fn assert_gfm_blocks(parsed: &MarkdownSubset) {
    assert!(contains_block(parsed, |it| matches!(
        it,
        MarkdownBlock::Heading(_)
    )));
    assert!(contains_block(parsed, |it| matches!(
        it,
        MarkdownBlock::CodeBlock(_)
    )));
    assert!(contains_block(parsed, |it| matches!(
        it,
        MarkdownBlock::BlockQuote(_)
    )));
    assert!(contains_block(parsed, |it| matches!(
        it,
        MarkdownBlock::List(_)
    )));
    assert!(contains_block(parsed, |it| matches!(
        it,
        MarkdownBlock::Table(_)
    )));
}

fn assert_gfm_inline_segments(parsed: &MarkdownSubset) {
    assert!(contains_inline_segment(parsed, |it| matches!(
        it,
        InlineSegment::Strong(_)
    )));
    assert!(contains_inline_segment(parsed, |it| matches!(
        it,
        InlineSegment::Emphasis(_)
    )));
    assert!(contains_inline_segment(parsed, |it| matches!(
        it,
        InlineSegment::Strikethrough(_)
    )));
    assert!(contains_inline_segment(parsed, |it| matches!(
        it,
        InlineSegment::Link { .. }
    )));
}

fn assert_gfm_task_items(parsed: &MarkdownSubset) {
    assert!(contains_checked_task(parsed, true));
    assert!(contains_checked_task(parsed, false));
}

fn contains_block(parsed: &MarkdownSubset, predicate: impl Fn(&MarkdownBlock) -> bool) -> bool {
    parsed.blocks.iter().any(predicate)
}

fn text_has_segment(text: &super::TextBlock, predicate: impl Fn(&InlineSegment) -> bool) -> bool {
    text.segments.iter().any(predicate)
}

fn contains_checked_task(parsed: &MarkdownSubset, checked: bool) -> bool {
    parsed.blocks.iter().any(|block| match block {
        MarkdownBlock::List(list) => list.items.iter().any(|it| it.checked == Some(checked)),
        _ => false,
    })
}
