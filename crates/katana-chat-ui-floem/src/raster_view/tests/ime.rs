use floem::event::Event;
use katana_chat_ui::kuc::{ChatUiRuntimeCompositionPhase, ChatUiRuntimeInput};

use super::event_input;

const PREEDIT: &str = "日本😀";

#[test]
fn native_ime_preedit_preserves_utf8_caret_offsets() {
    assert_eq!(preedit_caret(Some((3, 3))), 3);
    assert_eq!(preedit_caret(None), PREEDIT.len());
}

fn preedit_caret(cursor: Option<(usize, usize)>) -> usize {
    match event_input(&preedit(cursor), false) {
        Some(ChatUiRuntimeInput::ImeComposition {
            phase: ChatUiRuntimeCompositionPhase::Update,
            preedit,
            caret,
        }) if preedit == PREEDIT => caret,
        _ => 0,
    }
}

fn preedit(cursor: Option<(usize, usize)>) -> Event {
    Event::ImePreedit {
        text: PREEDIT.to_string(),
        cursor,
    }
}
