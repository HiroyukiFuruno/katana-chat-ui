use super::{
    ICON_BUTTON_SIZE,
    icons::{PrimaryIcon, button_at, primary_button_at},
};
use crate::view::EguiChatController;
use eframe::egui;
use katana_chat_ui::ChatUiSurface;

mod layout;
mod usage;
mod vendor;

use layout::{
    ControlRects, compact_control_width, use_compact_controls, wide_controls_left,
    wide_controls_width,
};
use usage::usage_meter_at;

pub(super) struct EguiComposerControls;

impl EguiComposerControls {
    pub(super) fn render(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let control_count = surface.vendor_bar.controls.len();
        if use_compact_controls(ui.available_width(), control_count) {
            Self::compact(ui, surface, controller);
            return;
        }
        Self::wide(ui, surface, controller);
    }

    fn compact(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        Self::compact_vendor_controls(ui, surface, controller);
        ui.add_space(layout::CONTROL_ROW_GAP);
        Self::action_controls(ui, surface, controller);
    }

    fn compact_vendor_controls(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let control_count = surface.vendor_bar.controls.len();
        if control_count == 0 {
            return;
        }
        let control_width = compact_control_width(ui.available_width(), control_count);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing =
                egui::vec2(layout::CONTROL_ROW_GAP, layout::CONTROL_ROW_GAP);
            for control in &surface.vendor_bar.controls {
                let rect = allocate_control_rect(ui, control_width);
                vendor::render_vendor_control(ui, rect, control, controller);
            }
        });
    }

    fn wide(ui: &mut egui::Ui, surface: &ChatUiSurface, controller: &mut impl EguiChatController) {
        let (row_rect, attach_rect, primary_rect, usage_rect) = Self::action_controls_rects(ui);
        Self::attach(ui, surface, controller, attach_rect);
        Self::primary_action(ui, surface, controller, primary_rect);
        usage_meter_at(ui, surface, usage_rect);
        Self::wide_vendor_controls(ui, surface, controller, attach_rect, usage_rect, row_rect);
    }

    fn action_controls(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let (_, attach_rect, primary_rect, usage_rect) = Self::action_controls_rects(ui);
        Self::attach(ui, surface, controller, attach_rect);
        Self::primary_action(ui, surface, controller, primary_rect);
        usage_meter_at(ui, surface, usage_rect);
    }

    fn action_controls_rects(ui: &mut egui::Ui) -> ControlRects {
        let (row_rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), ICON_BUTTON_SIZE),
            egui::Sense::hover(),
        );
        let attach_rect = egui::Rect::from_min_size(
            row_rect.left_top(),
            egui::vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
        );
        let primary_rect = egui::Rect::from_min_size(
            egui::pos2(row_rect.right() - ICON_BUTTON_SIZE, row_rect.top()),
            egui::vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE),
        );
        let usage_rect = egui::Rect::from_min_size(
            egui::pos2(
                primary_rect.left() - layout::CONTROL_ROW_GAP - layout::USAGE_SIZE,
                row_rect.top(),
            ),
            egui::vec2(layout::USAGE_SIZE, ICON_BUTTON_SIZE),
        );
        (row_rect, attach_rect, primary_rect, usage_rect)
    }

    fn attach(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
        rect: egui::Rect,
    ) {
        if button_at(ui, rect, "📎", &surface.composer.attach_label).clicked() {
            controller.attach();
        }
    }

    fn wide_vendor_controls(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
        attach_rect: egui::Rect,
        usage_rect: egui::Rect,
        row_rect: egui::Rect,
    ) {
        let control_count = surface.vendor_bar.controls.len();
        if control_count == 0 {
            return;
        }
        let controls_width = wide_controls_width(attach_rect, usage_rect, control_count);
        let left = wide_controls_left(row_rect, attach_rect, usage_rect, controls_width);
        let control_width = layout::row_control_width(controls_width, control_count);
        for (index, control) in surface.vendor_bar.controls.iter().enumerate() {
            let x = left + ((control_width + layout::CONTROL_ROW_GAP) * index as f32);
            let rect = egui::Rect::from_min_size(
                egui::pos2(x, row_rect.top()),
                egui::vec2(control_width, row_rect.height()),
            );
            vendor::render_vendor_control(ui, rect, control, controller);
        }
    }

    fn primary_action(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
        rect: egui::Rect,
    ) {
        let icon = primary_icon(surface);
        let tooltip = primary_tooltip(surface);
        if !primary_button_at(ui, rect, icon, tooltip).clicked() {
            return;
        }
        let _ = if surface.composer.stop_enabled {
            controller.stop()
        } else {
            controller.submit()
        };
    }
}

fn allocate_control_rect(ui: &mut egui::Ui, width: f32) -> egui::Rect {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, ICON_BUTTON_SIZE), egui::Sense::hover());
    rect
}

fn primary_icon(surface: &ChatUiSurface) -> PrimaryIcon {
    if surface.composer.stop_enabled {
        return PrimaryIcon::Stop;
    }
    PrimaryIcon::Send
}

fn primary_tooltip(surface: &ChatUiSurface) -> &str {
    if surface.composer.stop_enabled {
        return &surface.composer.stop_label;
    }
    &surface.composer.send_label
}

#[cfg(test)]
mod tests {
    #[test]
    fn compact_controls_are_used_before_buttons_overlap() {
        assert!(super::layout::use_compact_controls(320.0, 3));
        assert!(!super::layout::use_compact_controls(900.0, 3));
        assert!(!super::layout::use_compact_controls(320.0, 0));
    }

    #[test]
    fn compact_control_width_uses_available_row_width() {
        let width = super::layout::compact_control_width(320.0, 2);

        assert_eq!(width, 155.0);
    }
}
