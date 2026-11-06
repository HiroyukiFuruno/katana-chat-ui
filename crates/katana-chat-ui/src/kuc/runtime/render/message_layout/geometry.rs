use super::{ChatUiRuntimeError, MessageLayoutIndex, missing_message_geometry};
use katana_ui_core::render_model::{UiDimension, UiNode, UiNodeKind, UiPosition};
use std::collections::HashMap;

impl MessageLayoutIndex {
    pub(super) fn visit(
        &mut self,
        node: &UiNode,
        cards: &HashMap<String, usize>,
        parent_message: Option<usize>,
    ) -> Result<(), ChatUiRuntimeError> {
        let node_id = node.id().as_str();
        let card_index = cards.get(node_id).copied();
        let message_index = card_index.or(parent_message);
        let skip_children = self.record_geometry(node, node_id, card_index, message_index)?;
        if skip_children {
            return Ok(());
        }
        for child in node.children() {
            self.visit(child, cards, message_index)?;
        }
        Ok(())
    }

    fn record_geometry(
        &mut self,
        node: &UiNode,
        node_id: &str,
        card_index: Option<usize>,
        message_index: Option<usize>,
    ) -> Result<bool, ChatUiRuntimeError> {
        self.record_row_height(node, node_id);
        if node_id == "chat.frame.thread" {
            self.thread_top = dimension_px(&node.props().common.margin.top);
        }
        if let Some(index) = card_index {
            self.record_card(node, node_id, index)?;
            return Ok(false);
        }
        if let Some(index) = message_index {
            if is_out_of_flow(node) {
                return Ok(true);
            }
            self.flow_nodes.insert(node_id.to_owned(), index);
            return Ok(node.kind() == UiNodeKind::ScrollArea);
        }
        Ok(false)
    }

    fn record_row_height(&mut self, node: &UiNode, node_id: &str) {
        let Some(index) = self.rows.get(node_id).copied() else {
            return;
        };
        let Some(height) = dimension_px(&node.props().common.height) else {
            return;
        };
        self.row_heights[index] = Some(height);
    }

    fn record_card(
        &mut self,
        node: &UiNode,
        node_id: &str,
        index: usize,
    ) -> Result<(), ChatUiRuntimeError> {
        let message_id = node_id.strip_prefix("chat.message.").unwrap_or(node_id);
        let padding = dimension_px(&node.props().common.padding.bottom)
            .ok_or_else(|| missing_message_geometry(message_id, "fixed card padding"))?;
        self.bottom_padding[index] = Some(padding);
        self.empty_cards[index] = node.children().is_empty();
        Ok(())
    }
}

fn is_out_of_flow(node: &UiNode) -> bool {
    matches!(
        node.props().common.position,
        UiPosition::Absolute | UiPosition::Fixed | UiPosition::Sticky
    )
}

fn dimension_px(dimension: &UiDimension) -> Option<u32> {
    match dimension {
        UiDimension::Px(value) => Some(u32::from(*value)),
        _ => None,
    }
}
