mod inline;
mod nodes;

use crate::MarkdownBlock;
use katana_ui_core::render_model::UiNode;

pub(super) fn markdown_nodes(blocks: &[MarkdownBlock], node_prefix: &str) -> Vec<UiNode> {
    nodes::markdown_nodes(blocks, node_prefix)
}

#[cfg(test)]
mod tests;
