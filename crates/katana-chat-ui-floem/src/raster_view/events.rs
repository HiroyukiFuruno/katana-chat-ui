use floem::{
    event::Event,
    keyboard::{Key, NamedKey},
};
use katana_chat_ui::kuc::{ChatUiRuntimeCompositionPhase, ChatUiRuntimeInput, ChatUiRuntimeKey};

pub(super) fn event_input(event: &Event, active: bool) -> Option<ChatUiRuntimeInput> {
    match event {
        Event::PointerDown(event) if event.button.is_primary() => {
            Some(ChatUiRuntimeInput::PointerPress {
                x: event.pos.x as f32,
                y: event.pos.y as f32,
                extend_selection: event.modifiers.shift(),
            })
        }
        Event::PointerMove(event) if active => Some(ChatUiRuntimeInput::PointerDrag {
            x: event.pos.x as f32,
            y: event.pos.y as f32,
        }),
        Event::PointerUp(event) if event.button.is_primary() && active => {
            Some(ChatUiRuntimeInput::PointerRelease)
        }
        Event::PointerWheel(event) => Some(ChatUiRuntimeInput::Scroll {
            x: event.pos.x as f32,
            y: event.pos.y as f32,
            delta_x: event.delta.x.round() as i32,
            delta_y: event.delta.y.round() as i32,
        }),
        Event::KeyDown(event) => key_input(event),
        _ => composition_input(event),
    }
}

fn composition_input(event: &Event) -> Option<ChatUiRuntimeInput> {
    match event {
        Event::ImeEnabled => Some(composition(
            ChatUiRuntimeCompositionPhase::Start,
            String::new(),
            0,
        )),
        Event::ImePreedit { text, cursor } => Some(composition(
            ChatUiRuntimeCompositionPhase::Update,
            text.clone(),
            cursor.map_or_else(|| text.len(), |(_, end)| end),
        )),
        Event::ImeCommit(text) => Some(ChatUiRuntimeInput::ImeCommit(text.clone())),
        Event::ImeDisabled => Some(ChatUiRuntimeInput::ImeCancel),
        Event::FocusLost => Some(ChatUiRuntimeInput::FocusLost),
        _ => None,
    }
}

pub(super) fn is_paste(event: &Event) -> bool {
    let Event::KeyDown(event) = event else {
        return false;
    };
    matches!(&event.key.logical_key, Key::Character(value)
        if primary(event) && value.eq_ignore_ascii_case("v"))
}

fn composition(
    phase: ChatUiRuntimeCompositionPhase,
    preedit: String,
    caret: usize,
) -> ChatUiRuntimeInput {
    ChatUiRuntimeInput::ImeComposition {
        phase,
        preedit,
        caret,
    }
}

fn key_input(event: &floem::keyboard::KeyEvent) -> Option<ChatUiRuntimeInput> {
    key_input_parts(
        &event.key.logical_key,
        event.key.text.as_deref(),
        event.modifiers.shift(),
        primary(event),
        event.key.repeat,
    )
}

pub(super) fn key_input_parts(
    key: &Key,
    text: Option<&str>,
    shift: bool,
    command: bool,
    repeat: bool,
) -> Option<ChatUiRuntimeInput> {
    if repeat && command && matches!(key, Key::Named(NamedKey::Enter)) {
        return None;
    }
    let key = match key {
        Key::Named(NamedKey::Enter) => Some(ChatUiRuntimeKey::Enter { shift, command }),
        Key::Named(NamedKey::Backspace) => Some(ChatUiRuntimeKey::Backspace),
        Key::Named(NamedKey::Delete) => Some(ChatUiRuntimeKey::Delete),
        Key::Named(NamedKey::ArrowLeft) => Some(ChatUiRuntimeKey::ArrowLeft),
        Key::Named(NamedKey::ArrowRight) => Some(ChatUiRuntimeKey::ArrowRight),
        Key::Named(NamedKey::Home) => Some(ChatUiRuntimeKey::Home),
        Key::Named(NamedKey::End) => Some(ChatUiRuntimeKey::End),
        Key::Character(value) if command && value.eq_ignore_ascii_case("c") => {
            return Some(ChatUiRuntimeInput::ClipboardCopy);
        }
        Key::Character(value) if command && value.eq_ignore_ascii_case("x") => {
            return Some(ChatUiRuntimeInput::ClipboardCut);
        }
        Key::Character(value) if command && value.eq_ignore_ascii_case("a") => {
            Some(ChatUiRuntimeKey::CommandA)
        }
        Key::Character(_) if !command => {
            return text.map(|value| ChatUiRuntimeInput::Text(value.to_string()));
        }
        _ => None,
    };
    key.map(ChatUiRuntimeInput::Key)
}

fn primary(event: &floem::keyboard::KeyEvent) -> bool {
    event.modifiers.meta() || event.modifiers.control()
}
