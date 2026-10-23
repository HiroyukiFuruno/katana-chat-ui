use eframe::egui;

use super::{VIEWPORT_HEIGHT, VIEWPORT_WIDTH};

pub(super) fn raw_input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        )),
        max_texture_side: Some(4096),
        events,
        ..Default::default()
    }
}

pub(super) fn pointer_click(position: egui::Pos2) -> Vec<egui::Event> {
    vec![
        pointer_button(position, true),
        pointer_button(position, false),
    ]
}

pub(super) fn enter(command: bool, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            command,
            mac_cmd: command,
            ..Default::default()
        },
    }
}

fn pointer_button(position: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: position,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    }
}
