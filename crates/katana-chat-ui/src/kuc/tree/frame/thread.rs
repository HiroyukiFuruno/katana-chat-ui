use super::{
    ChatFrameLayout, ChatUiActionError, ChatUiSurface, HEADER_HEIGHT, content, output_handoff,
    positioned,
};
use katana_ui_core::render_model::{UiNode, UiScrollAreaProps};

pub(super) fn thread(
    surface: &ChatUiSurface,
    layout: ChatFrameLayout,
    scroll_y: u32,
    message_heights: &[u16],
) -> Result<UiNode, ChatUiActionError> {
    let row_heights = content::effective_message_heights(
        &surface.message_list.messages,
        layout.body_width,
        message_heights,
    );
    let content_height =
        content::thread_content_height(&row_heights).max(u32::from(layout.thread_height));
    let mut content = content::thread_for_width(
        &surface.message_list.messages,
        layout.body_width,
        &row_heights,
    )?
    .scroll_area(thread_scroll_area(layout, content_height, scroll_y));
    if !surface.output_handoff.outputs.is_empty() {
        content = content.child(output_handoff(surface)?);
    }
    Ok(positioned(
        content,
        "chat.frame.thread",
        layout,
        HEADER_HEIGHT,
        layout.thread_height,
    ))
}

fn thread_scroll_area(
    layout: ChatFrameLayout,
    content_height: u32,
    scroll_y: u32,
) -> UiScrollAreaProps {
    let viewport_height = u32::from(layout.thread_height);
    UiScrollAreaProps {
        viewport_width: u32::from(layout.body_width),
        viewport_height,
        content_width: u32::from(layout.body_width),
        content_height,
        offset_y: scroll_y.min(content_height.saturating_sub(viewport_height)),
        ..UiScrollAreaProps::default()
    }
}
