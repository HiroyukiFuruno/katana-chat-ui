use crate::widget::styles;
use floem::prelude::*;
use katana_chat_ui::{Attachment, ChatUiSurface};

const ATTACHMENT_PADDING_X: f64 = 10.0;
const ATTACHMENT_PADDING_Y: f64 = 4.0;
const ATTACHMENT_RADIUS: f64 = 10.0;
const ATTACHMENT_CONTENT_GAP: f64 = 6.0;
const REMOVE_ICON_SIZE: f64 = 12.0;
const REMOVE_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" data-kcu-icon="remove-attachment" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 6l12 12"/><path d="M18 6 6 18"/></svg>"#;

pub(super) struct ComposerAttachmentTray;

impl ComposerAttachmentTray {
    pub(super) fn render<OnRemove>(
        surface: RwSignal<ChatUiSurface>,
        on_remove: OnRemove,
    ) -> impl IntoView
    where
        OnRemove: Fn(usize) + Copy + 'static,
    {
        dyn_stack(
            move || {
                let composer = surface.get().composer;
                attachment_items(composer.attachments, composer.remove_attachment_label)
            },
            |item| item.index,
            move |item| attachment_chip(item, on_remove),
        )
        .style(|style| style.width_full().gap(styles::PANEL_GAP).items_center())
    }
}

#[derive(Clone)]
struct AttachmentItem {
    index: usize,
    attachment: Attachment,
    remove_label: String,
}

fn attachment_items(attachments: Vec<Attachment>, remove_label: String) -> Vec<AttachmentItem> {
    attachments
        .into_iter()
        .enumerate()
        .map(|(index, attachment)| AttachmentItem {
            index,
            attachment,
            remove_label: remove_label.clone(),
        })
        .collect()
}

fn attachment_chip<OnRemove>(item: AttachmentItem, on_remove: OnRemove) -> impl IntoView
where
    OnRemove: Fn(usize) + Copy + 'static,
{
    let tooltip = item.remove_label.clone();
    h_stack((
        text(attachment_label(&item.attachment)),
        svg(REMOVE_ICON).style(|style| {
            style
                .size(REMOVE_ICON_SIZE, REMOVE_ICON_SIZE)
                .color(styles::COLOR_MUTED)
        }),
    ))
    .on_click_stop(move |_| on_remove(item.index))
    .tooltip(move || text(tooltip.clone()))
    .style(|style| {
        style
            .padding_horiz(ATTACHMENT_PADDING_X)
            .padding_vert(ATTACHMENT_PADDING_Y)
            .gap(ATTACHMENT_CONTENT_GAP)
            .items_center()
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(ATTACHMENT_RADIUS)
            .font_size(styles::FONT_META)
            .color(styles::COLOR_TEXT)
            .background(styles::COLOR_PANEL)
    })
}

fn attachment_label(attachment: &Attachment) -> String {
    match attachment {
        Attachment::Text(attachment) => attachment.label.clone(),
        Attachment::FileResource(attachment) => file_label(&attachment.uri),
        Attachment::ImageResource(attachment) => attachment.mime_type.clone(),
        Attachment::Unsupported(attachment) => attachment.label.clone(),
    }
}

fn file_label(uri: &str) -> String {
    uri.rsplit('/')
        .next()
        .filter(|it| !it.is_empty())
        .unwrap_or(uri)
        .to_string()
}
