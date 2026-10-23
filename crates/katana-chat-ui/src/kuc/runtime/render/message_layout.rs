use super::super::types::ChatUiRuntimeError;
use super::hit_bottom;
use crate::ChatUiSurface;
use katana_ui_core::raster_host::UiTreeNodeHit;
use katana_ui_core::render_model::{UiDimension, UiNode, UiNodeKind, UiPosition};
use std::collections::HashMap;

pub(super) fn measure(
    root: &UiNode,
    surface: &ChatUiSurface,
    minimum_heights: &[u16],
    hits: &[UiTreeNodeHit],
) -> Result<Vec<u16>, ChatUiRuntimeError> {
    MessageLayoutIndex::new(root, surface)?.measured_heights(surface, minimum_heights, hits)
}

struct MessageLayoutIndex {
    rows: HashMap<String, usize>,
    row_heights: Vec<Option<u32>>,
    flow_nodes: HashMap<String, usize>,
    bottom_padding: Vec<Option<u32>>,
    empty_cards: Vec<bool>,
    thread_top: Option<u32>,
}

impl MessageLayoutIndex {
    fn new(root: &UiNode, surface: &ChatUiSurface) -> Result<Self, ChatUiRuntimeError> {
        let (rows, cards) = message_node_indexes(surface);
        let mut index = Self {
            rows,
            row_heights: vec![None; surface.message_list.messages.len()],
            flow_nodes: HashMap::new(),
            bottom_padding: vec![None; surface.message_list.messages.len()],
            empty_cards: vec![false; surface.message_list.messages.len()],
            thread_top: None,
        };
        index.visit(root, &cards, None)?;
        index.validate_geometry(surface)?;
        Ok(index)
    }

    fn validate_geometry(&self, surface: &ChatUiSurface) -> Result<(), ChatUiRuntimeError> {
        for (message, row_height) in surface.message_list.messages.iter().zip(&self.row_heights) {
            let id = message.id.to_string();
            if row_height.is_none() {
                return Err(missing_message_geometry(&id, "assigned row height"));
            }
        }
        for (message, padding) in surface
            .message_list
            .messages
            .iter()
            .zip(&self.bottom_padding)
        {
            if padding.is_none() {
                return Err(missing_message_geometry(
                    &message.id.to_string(),
                    "card node",
                ));
            }
        }
        if self.thread_top.is_none() {
            return Err(missing_message_geometry("thread", "assigned thread top"));
        }
        Ok(())
    }

    fn visit(
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

    fn measured_heights(
        &self,
        surface: &ChatUiSurface,
        minimum_heights: &[u16],
        hits: &[UiTreeNodeHit],
    ) -> Result<Vec<u16>, ChatUiRuntimeError> {
        let row_tops = self.row_tops(surface)?;
        let content_bottoms = self.content_bottoms(minimum_heights.len(), hits);
        surface
            .message_list
            .messages
            .iter()
            .enumerate()
            .map(|(index, message)| {
                self.measured_row_height(
                    message.id,
                    index,
                    row_tops[index],
                    content_bottoms[index],
                    minimum_heights[index],
                )
            })
            .collect()
    }

    fn row_tops(&self, surface: &ChatUiSurface) -> Result<Vec<u32>, ChatUiRuntimeError> {
        let mut next_top = self
            .thread_top
            .ok_or_else(|| missing_message_geometry("thread", "assigned thread top"))?;
        surface
            .message_list
            .messages
            .iter()
            .zip(&self.row_heights)
            .map(|(message, height)| {
                let row_top = next_top;
                next_top = next_top.saturating_add(height.ok_or_else(|| {
                    missing_message_geometry(&message.id.to_string(), "assigned row height")
                })?);
                Ok(row_top)
            })
            .collect()
    }

    fn content_bottoms(&self, count: usize, hits: &[UiTreeNodeHit]) -> Vec<Option<u32>> {
        let mut bottoms = vec![None::<u32>; count];
        for hit in hits {
            if let Some(index) = self.flow_nodes.get(hit.node_id.as_str()) {
                let bottom = hit_bottom(hit.rect.y, hit.rect.height);
                bottoms[*index] =
                    Some(bottoms[*index].map_or(bottom, |current| current.max(bottom)));
            }
        }
        bottoms
    }

    fn measured_row_height(
        &self,
        message_id: u64,
        index: usize,
        row_top: u32,
        content_bottom: Option<u32>,
        minimum_height: u16,
    ) -> Result<u16, ChatUiRuntimeError> {
        let id = message_id.to_string();
        let Some(content_bottom) = content_bottom else {
            if self.empty_cards[index] {
                return Ok(minimum_height);
            }
            return Err(missing_message_geometry(&id, "rendered content bounds"));
        };
        let padding = self.bottom_padding[index]
            .ok_or_else(|| missing_message_geometry(&id, "card bottom padding"))?;
        let required = content_bottom
            .saturating_sub(row_top)
            .saturating_add(padding);
        Ok(u16::try_from(required)
            .unwrap_or(u16::MAX)
            .max(minimum_height))
    }
}

fn message_node_indexes(
    surface: &ChatUiSurface,
) -> (HashMap<String, usize>, HashMap<String, usize>) {
    let rows = surface
        .message_list
        .messages
        .iter()
        .enumerate()
        .map(|(index, message)| (format!("chat.message.{}.row", message.id), index))
        .collect();
    let cards = surface
        .message_list
        .messages
        .iter()
        .enumerate()
        .map(|(index, message)| (format!("chat.message.{}", message.id), index))
        .collect();
    (rows, cards)
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

fn missing_message_geometry(message_id: &str, component: &str) -> ChatUiRuntimeError {
    ChatUiRuntimeError::MissingMessageGeometry {
        message_id: message_id.to_owned(),
        component: component.to_owned(),
    }
}
