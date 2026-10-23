use eframe::egui;
use katana_chat_ui::kuc::{
    ChatUiRuntimeCompositionPhase as Phase, ChatUiRuntimeInput as Input, ChatUiRuntimeKey as Key,
};

pub(super) fn collect(ui: &egui::Ui, response: &egui::Response) -> Vec<Input> {
    let mut events = Vec::new();
    for event in ui.input(|input| input.events.clone()) {
        if let Some(input) = pointer(&event, response) {
            events.push(input);
        } else if response.has_focus() || response.clicked() {
            events.extend(keyboard(event));
        }
    }
    if response.lost_focus() {
        events.push(Input::FocusLost);
    }
    if response.hovered() {
        let (delta, position) =
            ui.input(|input| (input.smooth_scroll_delta, input.pointer.hover_pos()));
        if let Some(position) = position.filter(|_| delta != egui::Vec2::ZERO) {
            let point = position - response.rect.min;
            events.push(Input::Scroll {
                x: point.x,
                y: point.y,
                delta_x: -delta.x as i32,
                delta_y: -delta.y as i32,
            });
        }
    }
    events
}

fn pointer(event: &egui::Event, response: &egui::Response) -> Option<Input> {
    match event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers,
        } if response.rect.contains(*pos) || response.dragged() || response.drag_stopped() => {
            let point = *pos - response.rect.min;
            Some(if *pressed {
                Input::PointerPress {
                    x: point.x,
                    y: point.y,
                    extend_selection: modifiers.shift,
                }
            } else {
                Input::PointerRelease
            })
        }
        egui::Event::PointerMoved(pos) if response.dragged() => {
            let point = *pos - response.rect.min;
            Some(Input::PointerDrag {
                x: point.x,
                y: point.y,
            })
        }
        _ => None,
    }
}

fn keyboard(event: egui::Event) -> Option<Input> {
    match event {
        egui::Event::Text(text) => Some(Input::Text(text)),
        egui::Event::Paste(text) => Some(Input::ClipboardPaste(text)),
        egui::Event::Copy => Some(Input::ClipboardCopy),
        egui::Event::Cut => Some(Input::ClipboardCut),
        egui::Event::Ime(ime) => composition(ime),
        egui::Event::Key {
            key,
            pressed: true,
            repeat,
            modifiers,
            ..
        } => {
            if repeat && key == egui::Key::Enter && modifiers.command {
                return None;
            }
            key_input(key, modifiers).map(Input::Key)
        }
        _ => None,
    }
}

fn key_input(key: egui::Key, modifiers: egui::Modifiers) -> Option<Key> {
    match key {
        egui::Key::Enter => Some(Key::Enter {
            shift: modifiers.shift,
            command: modifiers.command,
        }),
        egui::Key::Backspace => Some(Key::Backspace),
        egui::Key::Delete => Some(Key::Delete),
        egui::Key::ArrowLeft => Some(Key::ArrowLeft),
        egui::Key::ArrowRight => Some(Key::ArrowRight),
        egui::Key::Home => Some(Key::Home),
        egui::Key::End => Some(Key::End),
        egui::Key::A if modifiers.command => Some(Key::CommandA),
        _ => None,
    }
}

fn composition(event: egui::ImeEvent) -> Option<Input> {
    match event {
        egui::ImeEvent::Preedit { text, .. } if text.is_empty() => Some(Input::ImeCancel),
        egui::ImeEvent::Preedit {
            text,
            active_range_chars,
        } => Some(Input::ImeComposition {
            phase: Phase::Update,
            caret: active_range_chars
                .map_or_else(|| text.len(), |range| char_byte_offset(&text, range.end)),
            preedit: text,
        }),
        egui::ImeEvent::Commit(text) => Some(Input::ImeCommit(text)),
        egui::ImeEvent::DeleteSurrounding { .. } => None,
        _ => None,
    }
}

fn char_byte_offset(text: &str, char_offset: usize) -> usize {
    text.char_indices()
        .nth(char_offset)
        .map_or_else(|| text.len(), |(byte_offset, _)| byte_offset)
}

#[cfg(test)]
mod tests;
