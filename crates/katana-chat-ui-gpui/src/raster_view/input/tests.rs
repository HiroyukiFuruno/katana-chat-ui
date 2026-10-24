use gpui::{KeyDownEvent, Keystroke, Modifiers};
use katana_chat_ui::{
    ChatSession, ChatUiSurface,
    kuc::{
        ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeFrame, ChatUiRuntimeInput,
        ChatUiRuntimeViewport,
    },
};

use super::utf16;

#[test]
fn utf16_ime_range_reaches_the_kuc_draft_effect() -> Result<(), String> {
    let draft = "A😀B";
    let range = utf16::checked_byte_range(draft, 1..3).ok_or("UTF-16 range was rejected")?;
    assert_eq!(range, 1..5);

    let surface = surface(draft);
    let mut runtime = ChatUiRuntime::light(&surface).map_err(|error| error.to_string())?;
    let frame = focused_frame(&mut runtime, &surface)?;
    let effects = runtime
        .dispatch(
            &frame,
            ChatUiRuntimeInput::ImeCommitRange {
                start_byte: range.start,
                end_byte: range.end,
                text: "語".into(),
            },
        )
        .map_err(|error| error.to_string())?;

    assert!(
        effects.iter().any(
            |effect| matches!(effect, ChatUiRuntimeEffect::DraftChanged(text) if text == "A語B")
        )
    );
    Ok(())
}

#[test]
fn invalid_utf16_ime_ranges_are_rejected_before_kuc_dispatch() {
    let draft = "A😀B";
    let reversed_selection = std::ops::Range { start: 3, end: 1 };
    assert_eq!(utf16::checked_byte_range(draft, reversed_selection), None);
    assert_eq!(utf16::checked_byte_range(draft, 0..5), None);
}

#[test]
fn utf16_text_slice_preserves_a_surrogate_pair() {
    assert_eq!(utf16::slice("A😀B", 1..3).as_deref(), Some("😀"));
}

#[test]
fn utf16_preedit_selection_uses_a_utf8_byte_caret() {
    let text = "日本😀";
    assert_eq!(utf16::composition_caret_byte(text, Some(1..1)), 3);
    assert_eq!(utf16::composition_caret_byte(text, Some(2..4)), text.len());
    assert_eq!(utf16::composition_caret_byte(text, None), text.len());
}

#[test]
fn held_command_enter_is_filtered_before_runtime_dispatch() {
    assert!(super::command_enter_repeat(&key_down(true, true)));
    assert!(!super::command_enter_repeat(&key_down(false, true)));
    assert!(!super::command_enter_repeat(&key_down(true, false)));
}

fn key_down(is_held: bool, command: bool) -> KeyDownEvent {
    let modifiers = if command {
        Modifiers::command()
    } else {
        Modifiers::none()
    };
    KeyDownEvent {
        keystroke: Keystroke {
            modifiers,
            key: "enter".to_string(),
            key_char: None,
        },
        is_held,
    }
}

fn focused_frame(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
) -> Result<ChatUiRuntimeFrame, String> {
    let viewport = ChatUiRuntimeViewport::new(640, 480, 1.0);
    let before = runtime
        .render(surface, viewport)
        .map_err(|error| error.to_string())?;
    let bounds = before.composer.bounds;
    runtime
        .dispatch(
            &before,
            ChatUiRuntimeInput::PointerPress {
                x: bounds
                    .x
                    .saturating_add_unsigned(bounds.width.saturating_sub(1))
                    as f32,
                y: bounds.y as f32 + 1.0,
                extend_selection: false,
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .render(surface, viewport)
        .map_err(|error| error.to_string())
}

fn surface(draft: &str) -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.draft_mut().set_text(draft);
    ChatUiSurface::from_render_model(&session.render_model())
}
