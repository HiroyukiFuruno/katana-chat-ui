use super::{ChatUiActionError, ChatUiMessageSurface, message_for_width, node};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiEdgeInsets, UiNode, UiNodeKind, UiPosition,
};

pub(crate) fn thread_for_width(
    messages: &[ChatUiMessageSurface],
    body_width: u16,
    message_heights: &[u16],
) -> Result<UiNode, ChatUiActionError> {
    let mut thread = super::thread_base();
    for (index, surface) in messages.iter().enumerate() {
        let height = message_heights
            .get(index)
            .copied()
            .unwrap_or_else(|| message_height(surface, body_width));
        thread = thread.child(message_row(surface, body_width, height)?);
    }
    Ok(thread)
}

pub(crate) fn minimum_message_heights(
    messages: &[ChatUiMessageSurface],
    body_width: u16,
) -> Vec<u16> {
    messages
        .iter()
        .map(|surface| message_height(surface, body_width))
        .collect()
}

pub(crate) fn effective_message_heights(
    messages: &[ChatUiMessageSurface],
    body_width: u16,
    measured_heights: &[u16],
) -> Vec<u16> {
    minimum_message_heights(messages, body_width)
        .iter()
        .enumerate()
        .map(|(index, minimum)| {
            measured_heights
                .get(index)
                .copied()
                .unwrap_or(*minimum)
                .max(*minimum)
        })
        .collect()
}

pub(crate) fn thread_content_height(message_heights: &[u16]) -> u32 {
    message_heights.iter().fold(0_u32, |height, row_height| {
        height.saturating_add(u32::from(*row_height))
    })
}

fn message_row(
    surface: &ChatUiMessageSurface,
    body_width: u16,
    height: u16,
) -> Result<UiNode, ChatUiActionError> {
    let row_id = format!("chat.message.{}.row", surface.id);
    let card = message_for_width(surface, body_width)?
        .width(UiDimension::px(message_width(surface, body_width)));
    let row = row(&row_id, body_width, height);
    if surface.alignment == crate::ChatUiMessageAlignment::Trailing {
        return Ok(row.child(trailing_card(card)));
    }
    Ok(row.child(card))
}

fn trailing_card(card: UiNode) -> UiNode {
    card.position(UiPosition::Absolute).margin(UiEdgeInsets {
        top: UiDimension::px(0),
        right: UiDimension::px(1),
        bottom: UiDimension::px(0),
        left: UiDimension::px(0),
    })
}

fn row(row_id: &str, width: u16, height: u16) -> UiNode {
    node::container(UiNodeKind::Stack, "", row_id).common(
        UiCommonProps::default()
            .width(UiDimension::px(width))
            .height(UiDimension::px(height)),
    )
}

fn message_width(surface: &ChatUiMessageSurface, body_width: u16) -> u16 {
    if surface.alignment == crate::ChatUiMessageAlignment::Trailing {
        return content_width(&surface.body, body_width);
    }
    if surface.body.trim().is_empty() {
        return auxiliary_width(surface, body_width).unwrap_or(body_width);
    }
    body_width
}

fn auxiliary_width(surface: &ChatUiMessageSurface, body_width: u16) -> Option<u16> {
    let mut width = surface
        .activity
        .as_ref()
        .map(|activity| label_width(&activity.label, body_width));
    let Some(thinking) = &surface.thinking else {
        return width;
    };
    width = Some(
        width
            .unwrap_or(0)
            .max(label_width(&thinking.label, body_width)),
    );
    if thinking.expanded {
        for entry in &thinking.entries {
            width = Some(width.unwrap_or(0).max(label_width(entry, body_width)));
        }
    }
    width
}

fn label_width(label: &str, body_width: u16) -> u16 {
    label
        .lines()
        .map(|line| content_width(line, body_width))
        .max()
        .unwrap_or_else(|| content_width(label, body_width))
}

fn content_width(content: &str, body_width: u16) -> u16 {
    let length = u16::try_from(content.chars().count()).unwrap_or(u16::MAX);
    let minimum = 64.min(body_width);
    length
        .saturating_mul(9)
        .saturating_add(32)
        .clamp(minimum, body_width)
}

fn message_height(surface: &ChatUiMessageSurface, body_width: u16) -> u16 {
    let width = message_width(surface, body_width);
    let lines = wrapped_line_count(&surface.body, width)
        .saturating_add(status_line_count(surface))
        .saturating_add(thinking_line_count(surface, width));
    let lines = u16::try_from(lines).unwrap_or(u16::MAX);
    lines.saturating_mul(20).saturating_add(24).max(44)
}

fn thinking_line_count(surface: &ChatUiMessageSurface, width: u16) -> usize {
    let Some(thinking) = &surface.thinking else {
        return 0;
    };
    let label_lines = wrapped_line_count(&thinking.label, width);
    if !thinking.expanded {
        return label_lines;
    }
    thinking.entries.iter().fold(label_lines, |lines, entry| {
        lines.saturating_add(wrapped_line_count(entry, width))
    })
}

fn wrapped_line_count(content: &str, width: u16) -> usize {
    let characters_per_line = usize::from(width / 8).max(1);
    content
        .lines()
        .fold(0_usize, |count, line| {
            let characters = line.chars().count().max(1);
            count.saturating_add(
                characters
                    .saturating_add(characters_per_line.saturating_sub(1))
                    .saturating_div(characters_per_line),
            )
        })
        .max(1)
}

fn status_line_count(surface: &ChatUiMessageSurface) -> usize {
    if surface.status_label.trim().is_empty() {
        return 0;
    }
    surface.status_label.lines().count().max(1)
}
