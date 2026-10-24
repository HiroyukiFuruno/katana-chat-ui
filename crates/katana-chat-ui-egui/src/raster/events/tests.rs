use super::{Input, Key, composition, keyboard};
use eframe::egui;

fn enter(command: bool, pressed: bool, repeat: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed,
        repeat,
        modifiers: egui::Modifiers {
            command,
            mac_cmd: command,
            ..Default::default()
        },
    }
}

#[test]
fn command_enter_repeat_cannot_dispatch_a_second_submission() {
    assert_eq!(
        keyboard(enter(true, true, false)),
        Some(Input::Key(Key::Enter {
            shift: false,
            command: true
        }))
    );
    assert_eq!(keyboard(enter(true, true, true)), None);
    assert_eq!(keyboard(enter(true, false, false)), None);
    assert_eq!(
        keyboard(enter(false, true, true)),
        Some(Input::Key(Key::Enter {
            shift: false,
            command: false
        }))
    );
}

#[test]
fn empty_preedit_cancels_composition() {
    assert_eq!(
        composition(egui::ImeEvent::Preedit {
            text: String::new(),
            active_range_chars: None,
        }),
        Some(Input::ImeCancel)
    );
}

#[test]
fn active_range_preserves_the_japanese_ime_caret() {
    assert_eq!(
        composition(egui::ImeEvent::Preedit {
            text: "日本語😀".to_string(),
            active_range_chars: Some(1..2),
        }),
        Some(Input::ImeComposition {
            phase: super::Phase::Update,
            preedit: "日本語😀".to_string(),
            caret: "日本".len(),
        })
    );
}

#[test]
fn active_range_preserves_the_emoji_ime_caret() {
    assert_eq!(
        composition(egui::ImeEvent::Preedit {
            text: "a😀b".to_string(),
            active_range_chars: Some(1..2),
        }),
        Some(Input::ImeComposition {
            phase: super::Phase::Update,
            preedit: "a😀b".to_string(),
            caret: "a😀".len(),
        })
    );
}

#[test]
fn ime_commit_preserves_japanese_text() {
    assert_eq!(
        composition(egui::ImeEvent::Commit("日本語".to_string())),
        Some(Input::ImeCommit("日本語".to_string()))
    );
}

#[test]
fn clipboard_os_event_payload_is_preserved() {
    assert_eq!(
        keyboard(egui::Event::Paste("本文\n続き".to_string())),
        Some(Input::ClipboardPaste("本文\n続き".to_string()))
    );
    assert_eq!(keyboard(egui::Event::Cut), Some(Input::ClipboardCut));
}
