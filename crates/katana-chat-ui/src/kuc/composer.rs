use super::{ChatUiAction, ChatUiActionError, node};
use crate::{Attachment, ChatUiComposerSurface, ChatUiUsageSurface, ChatUiVendorBarSurface};
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiDisplay, UiInteractionState, UiLayoutAxis, UiNode, UiNodeKind,
    UiPosition, UiTextAreaNewlineKey, UiTextAreaProps, UiTextAreaSubmitKey,
};

mod footer;

pub(super) const INPUT_HEIGHT: u16 = 88;
const MIN_COMPOSER_HEIGHT: u16 = 160;
pub(super) const DEFAULT_COMPOSER_WIDTH: u16 = 560;

pub(super) fn composer(
    surface: &ChatUiComposerSurface,
    vendor: &ChatUiVendorBarSurface,
    usage_surface: &ChatUiUsageSurface,
    open_dropdown: Option<&str>,
    available_width: u16,
) -> Result<UiNode, ChatUiActionError> {
    let composer = base_composer(surface);
    let composer = attachments(composer, surface)?;
    let composer = composer.child(footer::footer(
        surface,
        vendor,
        usage_surface,
        open_dropdown,
        available_width,
    )?);
    Ok(composer)
}

pub(super) fn height(control_count: usize, available_width: u16) -> u16 {
    INPUT_HEIGHT
        .saturating_add(footer::height(control_count, available_width))
        .max(MIN_COMPOSER_HEIGHT)
}

fn base_composer(surface: &ChatUiComposerSurface) -> UiNode {
    composer_panel().child(composer_input(surface))
}

fn composer_panel() -> UiNode {
    node::container(UiNodeKind::Stack, "", "chat.composer")
        .accessibility_label("Composer")
        .common(
            UiCommonProps::default()
                .display(UiDisplay::Flex)
                .layout_axis(UiLayoutAxis::Vertical)
                .position(UiPosition::Sticky),
        )
}

fn composer_input(surface: &ChatUiComposerSurface) -> UiNode {
    let input = UiNode::new(UiNodeKind::TextArea, &surface.placeholder)
        .stable_node_id("chat.composer.input")
        .placeholder(&surface.placeholder)
        .interaction(UiInteractionState {
            value: surface.text.clone(),
            ..UiInteractionState::default()
        })
        .text_area(UiTextAreaProps {
            ime_enabled: surface.ime_enabled,
            submit_key: UiTextAreaSubmitKey::ModEnter,
            newline_key: UiTextAreaNewlineKey::Enter,
            ..UiTextAreaProps::default()
        });
    node::container(UiNodeKind::Stack, "", "chat.composer.input-slot")
        .common(UiCommonProps::default().height(UiDimension::px(INPUT_HEIGHT)))
        .child(input)
}

fn attachments(
    mut composer: UiNode,
    surface: &ChatUiComposerSurface,
) -> Result<UiNode, ChatUiActionError> {
    for (index, attachment) in surface.attachments.iter().enumerate() {
        composer = composer.child(attachment_node(attachment, index, surface)?);
    }
    Ok(composer)
}

fn attachment_node(
    attachment: &Attachment,
    index: usize,
    surface: &ChatUiComposerSurface,
) -> Result<UiNode, ChatUiActionError> {
    let label = match attachment {
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
    };
    node::action(
        UiNodeKind::AttachmentChip,
        &label,
        &format!("chat.composer.attachment.{index}"),
        &ChatUiAction::RemoveAttachment { index },
    )
    .map(|node| node.accessibility_label(&surface.remove_attachment_label))
}
