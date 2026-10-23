use std::{cell::RefCell, rc::Rc};

use floem::reactive::RwSignal;
use katana_chat_ui::{
    ChatSession, ChatUiAction, ChatUiSurface, VendorUiState, kuc::ChatUiRuntimeViewport,
};

use super::super::{super::FloemKucCallbacks, RasterChatView};

pub(super) fn callbacks(
    actions: Rc<RefCell<Vec<ChatUiAction>>>,
    drafts: Rc<RefCell<Vec<String>>>,
) -> FloemKucCallbacks<impl Fn(ChatUiAction), impl Fn(String)> {
    FloemKucCallbacks {
        on_action: move |action| actions.borrow_mut().push(action),
        on_draft: move |draft| drafts.borrow_mut().push(draft),
    }
}

pub(super) fn assert_preview(
    view: &RasterChatView<impl Fn(ChatUiAction), impl Fn(String)>,
) -> Result<(), String> {
    let frame = view
        .frame
        .as_ref()
        .ok_or("Floem render did not retain a KUC frame")?;
    let image = view
        .image
        .as_ref()
        .ok_or("Floem render did not create an image")?;
    let bytes = image.data.as_ref();
    if bytes.len() != frame.rgba.pixels.len() * 4 {
        return Err("Floem image bytes differ from the KUC raster size".into());
    }
    if !bytes.chunks_exact(4).all(|pixel| pixel[3] == u8::MAX) {
        return Err("Floem image has a non-opaque KUC pixel".into());
    }
    Ok(())
}

pub(super) fn ready_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.draft_mut().set_text("submit this");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("katanagent")
            .with_available_vendors(vec!["katanagent".to_string(), "codex-cli".to_string()]),
    );
    ChatUiSurface::from_render_model(&session.render_model())
}

pub(super) fn viewport() -> ChatUiRuntimeViewport {
    ChatUiRuntimeViewport::new(640, 480, 1.0)
}

pub(super) fn view_with_draft(
    draft: &str,
) -> Result<RasterChatView<impl Fn(ChatUiAction), impl Fn(String)>, String> {
    RasterChatView::new(
        RwSignal::new(ready_surface()),
        RwSignal::new(draft.to_string()),
        1.0,
        FloemKucCallbacks {
            on_action: |_| {},
            on_draft: |_| {},
        },
    )
    .map_err(|error| error.to_string())
}
