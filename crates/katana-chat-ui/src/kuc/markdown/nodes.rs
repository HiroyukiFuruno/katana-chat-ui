use super::inline::text_node;
use crate::markdown::{ListBlock, TableBlock};
use crate::{ListKind, MarkdownBlock, TextBlock};
use katana_ui_core::render_model::{UiNode, UiNodeKind, UiTextSpan};

pub(super) fn markdown_nodes(blocks: &[MarkdownBlock], node_prefix: &str) -> Vec<UiNode> {
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| block_node(block, &format!("{node_prefix}.{index}")))
        .collect()
}

fn block_node(block: &MarkdownBlock, node_id: &str) -> UiNode {
    match block {
        MarkdownBlock::Heading(heading) => text_node(
            &heading.text,
            node_id,
            heading_role(heading.level),
            Vec::new(),
        )
        .style_class(format!("markdown-heading-level-{}", heading.level)),
        MarkdownBlock::Paragraph(text) => {
            text_node(text, node_id, "markdown-paragraph", Vec::new())
        }
        MarkdownBlock::BlockQuote(text) => quote_node(text, node_id),
        MarkdownBlock::List(list) => list_node(list, node_id),
        MarkdownBlock::Table(table) => table_node(table, node_id),
        MarkdownBlock::CodeBlock(code) => code_node(code, node_id),
    }
}

fn heading_role(level: u8) -> &'static str {
    match level {
        1 => "heading",
        2 => "heading-2",
        _ => "heading-3",
    }
}

fn quote_node(text: &TextBlock, node_id: &str) -> UiNode {
    UiNode::new(UiNodeKind::Panel, "")
        .stable_node_id(node_id)
        .style_class("markdown-blockquote")
        .child(text_node(
            text,
            &format!("{node_id}.text"),
            "markdown-blockquote",
            Vec::new(),
        ))
}

fn list_node(list: &ListBlock, node_id: &str) -> UiNode {
    let mut node = UiNode::new(UiNodeKind::List, "").stable_node_id(node_id);
    for (index, item) in list.items.iter().enumerate() {
        node = node.child(list_item_node(item, list, node_id, index));
    }
    node
}

fn list_item_node(item: &crate::ListItem, list: &ListBlock, node_id: &str, index: usize) -> UiNode {
    let item_id = format!("{node_id}.item.{index}");
    let mut node = UiNode::new(UiNodeKind::Row, "").stable_node_id(item_id.as_str());
    if let Some(checked) = item.checked {
        node = node.child(task_node(&item_id, checked));
    }
    node.child(text_node(
        &item.content,
        &format!("{item_id}.text"),
        "markdown-list-item",
        vec![UiTextSpan::plain(list_marker(list, index))],
    ))
}

fn task_node(item_id: &str, checked: bool) -> UiNode {
    UiNode::new(UiNodeKind::Checkbox, "")
        .stable_node_id(format!("{item_id}.task"))
        .checked(checked)
        .disabled(true)
}

fn list_marker(list: &ListBlock, index: usize) -> String {
    match list.kind {
        ListKind::Ordered { start } => format!("{}. ", start.saturating_add(index as u64)),
        ListKind::Unordered => "• ".to_string(),
    }
}

fn table_node(table: &TableBlock, node_id: &str) -> UiNode {
    let mut node = UiNode::new(UiNodeKind::Grid, "")
        .stable_node_id(node_id)
        .style_class("markdown-table");
    if !table.headers.is_empty() {
        node = node.child(table_row_node(
            &table.headers,
            node_id,
            "header",
            "markdown-table-header",
        ));
    }
    for (index, row) in table.rows.iter().enumerate() {
        node = node.child(table_row_node(
            row,
            node_id,
            &format!("row.{index}"),
            "markdown-table-cell",
        ));
    }
    node
}

fn table_row_node(cells: &[TextBlock], node_id: &str, row_id: &str, role: &str) -> UiNode {
    let row_node_id = format!("{node_id}.{row_id}");
    let mut node = UiNode::new(UiNodeKind::Row, "").stable_node_id(row_node_id.as_str());
    for (index, cell) in cells.iter().enumerate() {
        node = node.child(text_node(
            cell,
            &format!("{row_node_id}.cell.{index}"),
            role,
            Vec::new(),
        ));
    }
    node
}

fn code_node(code: &crate::CodeBlock, node_id: &str) -> UiNode {
    let language = code.language.as_deref().unwrap_or("plain");
    UiNode::new(UiNodeKind::Panel, "")
        .stable_node_id(node_id)
        .child(super::inline::code_text_node(
            &code.code,
            &format!("{node_id}.text"),
            language,
        ))
}
