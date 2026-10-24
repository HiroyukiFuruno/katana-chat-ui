use floem::Clipboard;
use katana_chat_ui::{
    ChatUiAction,
    kuc::{ChatUiRuntimeClipboard, ChatUiRuntimeInput},
};

use super::RasterChatView;

impl<OnAction, OnDraft> RasterChatView<OnAction, OnDraft>
where
    OnAction: Fn(ChatUiAction) + 'static,
    OnDraft: Fn(String) + 'static,
{
    pub(super) fn apply_clipboard(&mut self, effect: ChatUiRuntimeClipboard) {
        match effect {
            ChatUiRuntimeClipboard::Read => self.paste(),
            ChatUiRuntimeClipboard::Write(text) => {
                let _ = Clipboard::set_contents(text);
            }
        }
    }

    pub(super) fn paste(&mut self) {
        if let Ok(text) = Clipboard::get_contents() {
            self.dispatch(ChatUiRuntimeInput::ClipboardPaste(text));
        }
    }

    pub(super) fn set_ime_cursor(&self) {
        if let Some(frame) = &self.frame {
            let origin = self.id.layout_rect().origin();
            let caret = frame
                .composer
                .text_surface
                .preedit
                .as_ref()
                .map_or(frame.composer.text_surface.selection.caret, |it| it.caret);
            floem::action::set_ime_allowed(true);
            floem::action::set_ime_cursor_area(
                floem::kurbo::Point::new(
                    origin.x + f64::from(caret.x),
                    origin.y + f64::from(caret.y),
                ),
                floem::kurbo::Size::new(caret.width.max(1).into(), caret.height.max(1).into()),
            );
        }
    }
}
