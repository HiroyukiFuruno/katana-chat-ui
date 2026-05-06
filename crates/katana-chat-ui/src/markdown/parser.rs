use super::inline::InlineCollector;
use super::{
    CodeBlock, HeadingBlock, ListBlock, ListItem, ListKind, MarkdownBlock, MarkdownSubset,
    TableBlock, TextBlock,
};
use comrak::nodes::{ListType, Node, NodeValue};
use comrak::{Arena, Options, parse_document};

#[derive(Debug)]
pub struct MarkdownParser<'a> {
    source: &'a str,
}

impl<'a> MarkdownParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn parse(self) -> MarkdownSubset {
        let arena = Arena::new();
        let options = markdown_options();
        let root = parse_document(&arena, self.source, &options);
        MarkdownSubset {
            blocks: root.children().filter_map(block_from_node).collect(),
        }
    }
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.tagfilter = true;
    options.parse.smart = true;
    options
}

fn block_from_node(node: Node<'_>) -> Option<MarkdownBlock> {
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Heading(heading) => Some(MarkdownBlock::Heading(HeadingBlock {
            level: heading.level,
            text: InlineCollector::collect(node),
        })),
        NodeValue::Paragraph => Some(MarkdownBlock::Paragraph(InlineCollector::collect(node))),
        NodeValue::CodeBlock(code) => Some(MarkdownBlock::CodeBlock(CodeBlock {
            language: code_language(&code.info),
            code: code.literal,
        })),
        NodeValue::HtmlBlock(html) => {
            Some(MarkdownBlock::Paragraph(TextBlock::from_text(html.literal)))
        }
        NodeValue::BlockQuote => Some(MarkdownBlock::BlockQuote(InlineCollector::collect(node))),
        NodeValue::List(list) => Some(MarkdownBlock::List(list_block(node, list.list_type))),
        NodeValue::Table(_) => Some(MarkdownBlock::Table(table_block(node))),
        _ => None,
    }
}

fn code_language(info: &str) -> Option<String> {
    let trimmed = info.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.split_whitespace().next().map(ToString::to_string)
}

fn list_block(node: Node<'_>, list_type: ListType) -> ListBlock {
    let kind = match list_type {
        ListType::Ordered => ordered_kind(node),
        ListType::Bullet => ListKind::Unordered,
    };
    ListBlock {
        kind,
        items: node.children().filter_map(list_item).collect(),
    }
}

fn ordered_kind(node: Node<'_>) -> ListKind {
    let start = match &node.data.borrow().value {
        NodeValue::List(list) => list.start as u64,
        _ => 1,
    };
    ListKind::Ordered { start }
}

fn list_item(node: Node<'_>) -> Option<ListItem> {
    let checked = match &node.data.borrow().value {
        NodeValue::Item(_) => None,
        NodeValue::TaskItem(task) => Some(task.symbol.is_some()),
        _ => return None,
    };
    Some(ListItem {
        checked,
        content: InlineCollector::collect(node),
    })
}

fn table_block(node: Node<'_>) -> TableBlock {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    for row in node.children() {
        let cells = table_cells(row);
        if is_header_row(row) {
            headers = cells;
        } else {
            rows.push(cells);
        }
    }
    TableBlock { headers, rows }
}

fn table_cells(row: Node<'_>) -> Vec<TextBlock> {
    row.children().map(InlineCollector::collect).collect()
}

fn is_header_row(row: Node<'_>) -> bool {
    matches!(row.data.borrow().value, NodeValue::TableRow(true))
}
