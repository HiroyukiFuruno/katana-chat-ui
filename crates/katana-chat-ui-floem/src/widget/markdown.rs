use super::styles;
use floem::{AnyView, peniko::Color, prelude::*};
use katana_chat_ui::{InlineSegment, ListKind, MarkdownBlock, TableBlock, TextBlock};
mod compact;

const BLOCK_GAP: f64 = 8.0;
const CODE_PADDING: f64 = 10.0;
const CODE_RADIUS: f64 = 8.0;
const LIST_ITEM_GAP: f64 = 4.0;
const QUOTE_BORDER_WIDTH: f64 = 3.0;
const HEADING_LEVEL_ONE_SIZE: f64 = 24.0;
const HEADING_LEVEL_TWO_SIZE: f64 = 20.0;
const HEADING_LEVEL_REST_SIZE: f64 = 17.0;
const CODE_BACKGROUND: Color = Color::from_rgb8(242, 244, 247);

pub(super) struct FloemMarkdownView;

impl FloemMarkdownView {
    pub(super) fn render(blocks: Vec<MarkdownBlock>, fallback: String) -> AnyView {
        if blocks.is_empty() {
            return paragraph(TextBlock::from_text(fallback)).into_any();
        }
        v_stack_from_iter(blocks.into_iter().map(block_view))
            .style(|style| style.width_full().min_width(0.0).gap(BLOCK_GAP))
            .into_any()
    }

    pub(super) fn render_compact(blocks: Vec<MarkdownBlock>, fallback: String) -> AnyView {
        compact::CompactMarkdownView::render(blocks, fallback)
    }
}

fn block_view(block: MarkdownBlock) -> AnyView {
    match block {
        MarkdownBlock::Heading(block) => label_text(block.text)
            .style(move |style| style.font_size(heading_size(block.level)))
            .into_any(),
        MarkdownBlock::Paragraph(block) => paragraph(block).into_any(),
        MarkdownBlock::BlockQuote(block) => quote(block).into_any(),
        MarkdownBlock::CodeBlock(block) => code_block(block.code).into_any(),
        MarkdownBlock::List(block) => list_block(block.kind, block.items).into_any(),
        MarkdownBlock::Table(block) => table_block(block).into_any(),
    }
}

fn paragraph(block: TextBlock) -> impl IntoView {
    label_text(block).style(|style| style.width_full().min_width(0.0))
}

fn quote(block: TextBlock) -> impl IntoView {
    container(label_text(block)).style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .padding_left(CODE_PADDING)
            .border_left(QUOTE_BORDER_WIDTH)
            .border_color(styles::COLOR_BORDER)
            .color(styles::COLOR_MUTED)
    })
}

fn code_block(code: String) -> impl IntoView {
    container(label(move || code.clone())).style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .padding(CODE_PADDING)
            .border_radius(CODE_RADIUS)
            .background(CODE_BACKGROUND)
            .font_size(styles::FONT_BODY)
    })
}

fn list_block(kind: ListKind, items: Vec<katana_chat_ui::ListItem>) -> impl IntoView {
    v_stack_from_iter(items.into_iter().enumerate().map(move |(index, item)| {
        let marker = list_marker(&kind, index, item.checked);
        label(move || format!("{marker} {}", inline_text(&item.content)))
    }))
    .style(|style| style.width_full().min_width(0.0).gap(LIST_ITEM_GAP))
}

fn table_block(table: TableBlock) -> impl IntoView {
    let mut lines = Vec::new();
    if !table.headers.is_empty() {
        lines.push(row_text(&table.headers));
    }
    for row in table.rows {
        lines.push(row_text(&row));
    }
    code_block(lines.join("\n"))
}

fn label_text(block: TextBlock) -> impl IntoView {
    label(move || inline_text(&block)).style(|style| {
        style
            .max_width_pct(100.0)
            .min_width(0.0)
            .font_size(styles::FONT_BODY)
            .color(styles::COLOR_TEXT)
    })
}

fn inline_text(block: &TextBlock) -> String {
    let mut text = String::new();
    for segment in &block.segments {
        text.push_str(&segment_text(segment));
    }
    text
}

fn segment_text(segment: &InlineSegment) -> String {
    match segment {
        InlineSegment::Text(text)
        | InlineSegment::Emphasis(text)
        | InlineSegment::Strong(text)
        | InlineSegment::Strikethrough(text)
        | InlineSegment::InlineCode(text) => text.clone(),
        InlineSegment::Link { text, .. } => text.clone(),
        InlineSegment::Image { alt, destination } if alt.is_empty() => destination.clone(),
        InlineSegment::Image { alt, .. } => alt.clone(),
    }
}

fn list_marker(kind: &ListKind, index: usize, checked: Option<bool>) -> String {
    match checked {
        Some(true) => "[x]".to_string(),
        Some(false) => "[ ]".to_string(),
        None => match kind {
            ListKind::Ordered { start } => format!("{}.", start + index as u64),
            ListKind::Unordered => "-".to_string(),
        },
    }
}

fn row_text(cells: &[TextBlock]) -> String {
    cells
        .iter()
        .map(inline_text)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn heading_size(level: u8) -> f64 {
    match level {
        1 => HEADING_LEVEL_ONE_SIZE,
        2 => HEADING_LEVEL_TWO_SIZE,
        _ => HEADING_LEVEL_REST_SIZE,
    }
}

#[cfg(test)]
mod tests;
