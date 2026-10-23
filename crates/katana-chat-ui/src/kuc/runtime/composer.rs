mod frame;
mod text;

pub(super) use frame::frame;

use crate::ChatUiSurface;
use katana_ui_core::atom::{TextArea, TextAreaNewlineKey, TextAreaSubmitKey};
use katana_ui_core::render_model::UiTextSpan;
use katana_ui_core::text_surface::{TextSurface, TextSurfaceProps, TextSurfaceViewport};

pub(super) fn text_surface(surface: &ChatUiSurface) -> TextSurface {
    let input = TextArea::new("Chat composer")
        .stable_state_id("chat.composer.input")
        .value(&surface.composer.text)
        .placeholder(&surface.composer.placeholder)
        .ime_enabled(surface.composer.ime_enabled)
        .submit_key(TextAreaSubmitKey::ModEnter)
        .newline_key(TextAreaNewlineKey::Enter);
    TextSurface::new(TextSurfaceProps::new(
        input,
        vec![UiTextSpan::plain(&surface.composer.text)],
        TextSurfaceViewport::default(),
    ))
}
