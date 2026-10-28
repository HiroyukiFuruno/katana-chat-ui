use floem::Clipboard;
use katana_chat_ui::{
    ChatUiAction,
    kuc::{ChatUiRuntimeClipboard, ChatUiRuntimeInput},
};

use super::RasterChatView;

#[cfg(target_os = "macos")]
fn ime_cursor_position_y(origin_y: f64, caret_y: i32, caret_height: u32) -> f64 {
    // floem-winit 0.29.5 の Cocoa firstRect 変換が rect height を原点計算から落とすため、caret の下端を渡す。
    origin_y + f64::from(caret_y) + f64::from(caret_height.max(1))
}

#[cfg(not(target_os = "macos"))]
fn ime_cursor_position_y(origin_y: f64, caret_y: i32, _caret_height: u32) -> f64 {
    origin_y + f64::from(caret_y)
}

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
                    ime_cursor_position_y(origin.y, caret.y, caret.height),
                ),
                floem::kurbo::Size::new(caret.width.max(1).into(), caret.height.max(1).into()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ime_cursor_position_y;

    #[cfg(target_os = "macos")]
    #[test]
    fn ime_cursor_position_uses_the_caret_bottom_for_macos() {
        assert_eq!(705.0 - ime_cursor_position_y(0.0, 490, 20), 195.0);
        assert_eq!(ime_cursor_position_y(63.0, 490, 20), 573.0);
        assert_eq!(ime_cursor_position_y(0.0, 490, 0), 491.0);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn ime_cursor_position_preserves_non_macos_coordinates() {
        assert_eq!(ime_cursor_position_y(63.0, 490, 20), 553.0);
    }
}
