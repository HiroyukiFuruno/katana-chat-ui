use super::{ChatUiActionError, node};
use crate::{Attachment, ChatUiMessageSurface};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiDisplay, UiEdgeInsets, UiJustifyContent, UiLayoutAxis, UiNode,
    UiNodeKind, UiOverflow,
};

mod frame;

const MESSAGE_CHILD_INDENT: u16 = 16;
const MESSAGE_VERTICAL_PADDING: u16 = 12;

pub(super) use frame::{
    effective_message_heights, minimum_message_heights, thread_content_height, thread_for_width,
};

pub(super) fn thread(messages: &[ChatUiMessageSurface]) -> Result<UiNode, ChatUiActionError> {
    let mut thread = thread_base();
    for surface in messages {
        thread = thread.child(message_row(surface)?);
    }
    Ok(thread)
}

fn thread_base() -> UiNode {
    node::container(UiNodeKind::ScrollArea, "Conversation", "chat.thread").common(
        UiCommonProps::default()
            .display(UiDisplay::Flex)
            .layout_axis(UiLayoutAxis::Vertical)
            .overflow(UiOverflow::Auto),
    )
}

fn message_row(surface: &ChatUiMessageSurface) -> Result<UiNode, ChatUiActionError> {
    let node_id = format!("chat.message.{}.row", surface.id);
    Ok(node::container(UiNodeKind::Row, "", &node_id)
        .common(
            UiCommonProps::default()
                .display(UiDisplay::Flex)
                .layout_axis(UiLayoutAxis::Horizontal)
                .justify_content(message_justify_content(surface)),
        )
        .child(message(surface)?))
}

pub(super) fn message(surface: &ChatUiMessageSurface) -> Result<UiNode, ChatUiActionError> {
    message_for_width(surface, u16::MAX)
}

pub(crate) fn message_for_width(
    surface: &ChatUiMessageSurface,
    _body_width: u16,
) -> Result<UiNode, ChatUiActionError> {
    let node_id = format!("chat.message.{}", surface.id);
    let message = message_base(surface, &node_id);
    let message = message_body(message, surface, &node_id);
    let message = message_activity(message, surface, &node_id);
    let message = message_thinking(message, surface, &node_id);
    let message = message_attachments(message, surface, &node_id);
    message_outputs(message, surface)
}

fn message_base(surface: &ChatUiMessageSurface, node_id: &str) -> UiNode {
    let common = UiCommonProps::default()
        .padding(message_padding())
        .child_indent(MESSAGE_CHILD_INDENT);
    let message = node::container(UiNodeKind::Card, "", node_id)
        .accessibility_label(&surface.role_label)
        .common(common);
    if surface.status_label.trim().is_empty() {
        return message;
    }
    message.child(node::text(
        &surface.status_label,
        &format!("{node_id}.status"),
        "message-status",
    ))
}

fn message_padding() -> UiEdgeInsets {
    UiEdgeInsets {
        top: UiDimension::px(MESSAGE_VERTICAL_PADDING),
        right: UiDimension::px(0),
        bottom: UiDimension::px(MESSAGE_VERTICAL_PADDING),
        left: UiDimension::px(0),
    }
}

fn message_body(mut message: UiNode, surface: &ChatUiMessageSurface, node_id: &str) -> UiNode {
    for block in super::markdown::markdown_nodes(&surface.blocks, &format!("{node_id}.body")) {
        message = message.child(block.hit_outset_left(MESSAGE_CHILD_INDENT));
    }
    message
}

fn message_activity(message: UiNode, surface: &ChatUiMessageSurface, node_id: &str) -> UiNode {
    if !surface.body.trim().is_empty() {
        return message;
    }
    if let Some(activity) = &surface.activity {
        return message.child(node::text(
            &activity.label,
            &format!("{node_id}.activity"),
            "message-activity",
        ));
    }
    message
}

fn message_thinking(mut message: UiNode, surface: &ChatUiMessageSurface, node_id: &str) -> UiNode {
    if let Some(thinking) = &surface.thinking {
        message = message.child(
            node::text(
                &thinking.label,
                &format!("{node_id}.thinking"),
                "message-thinking",
            )
            .style_class(thinking_style_class(thinking)),
        );
        if thinking.expanded {
            message = message.child(node::text(
                &thinking.entries.join("\n"),
                &format!("{node_id}.thinking.entries"),
                "message-thinking-entries",
            ));
        }
    }
    message
}

fn message_attachments(
    mut message: UiNode,
    surface: &ChatUiMessageSurface,
    node_id: &str,
) -> UiNode {
    for (index, attachment) in surface.attachments.iter().enumerate() {
        message = message.child(node::text(
            &attachment_label(attachment),
            &format!("{node_id}.attachment.{index}"),
            "message-attachment",
        ));
    }
    message
}

fn message_outputs(
    mut message: UiNode,
    surface: &ChatUiMessageSurface,
) -> Result<UiNode, ChatUiActionError> {
    for (index, output) in surface.outputs.iter().enumerate() {
        message = message.child(super::output::output_node(
            output,
            &format!("chat.message.{}.output.{index}", surface.id),
        )?);
    }
    Ok(message)
}

fn thinking_style_class(surface: &crate::ChatUiThinkingSurface) -> &'static str {
    if surface.completed {
        "message-thinking-completed"
    } else {
        "message-thinking-streaming"
    }
}

fn message_justify_content(surface: &ChatUiMessageSurface) -> UiJustifyContent {
    match surface.alignment {
        crate::ChatUiMessageAlignment::Leading => UiJustifyContent::Start,
        crate::ChatUiMessageAlignment::Trailing => UiJustifyContent::End,
    }
}

fn attachment_label(attachment: &Attachment) -> String {
    match attachment {
        Attachment::Text(value) => format!("{}\n{}", value.label, value.text),
        Attachment::FileResource(value) => format!(
            "{} ({}, {} bytes)\n{}",
            value.uri, value.mime_type, value.size_bytes, value.text
        ),
        Attachment::ImageResource(value) => format!(
            "{} ({} bytes)\n{}",
            value.mime_type, value.size_bytes, value.data_ref
        ),
        Attachment::Unsupported(value) => format!("{}: {}", value.label, value.reason),
    }
}
