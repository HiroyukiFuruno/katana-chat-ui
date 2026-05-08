use crate::composer::ICON_BUTTON_SIZE;
use crate::view::{color, colors};
use eframe::egui;

pub(super) enum PrimaryIcon {
    Send,
    Stop,
}

pub(super) fn primary_button_at(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    icon: PrimaryIcon,
    tooltip: &str,
) -> egui::Response {
    let response = ui.put(
        rect,
        egui::Button::new("")
            .frame(false)
            .min_size(egui::vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE)),
    );
    let painter = ui.painter_at(rect);
    match icon {
        PrimaryIcon::Send => draw_send_icon(&painter, rect),
        PrimaryIcon::Stop => draw_stop_icon(&painter, rect),
    }
    response.on_hover_text(tooltip)
}

pub(super) fn button_at(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    label: &str,
    tooltip: &str,
) -> egui::Response {
    ui.put(
        rect,
        egui::Button::new(egui::RichText::new(label).color(color(colors().text)))
            .frame(false)
            .min_size(egui::vec2(ICON_BUTTON_SIZE, ICON_BUTTON_SIZE)),
    )
    .on_hover_text(tooltip)
}

fn draw_send_icon(painter: &egui::Painter, rect: egui::Rect) {
    let stroke = egui::Stroke::new(2.0, color(colors().text));
    let center = rect.center();
    let top = egui::pos2(center.x, center.y - 8.0);
    let bottom = egui::pos2(center.x, center.y + 8.0);
    painter.line_segment([bottom, top], stroke);
    painter.line_segment([top, egui::pos2(center.x - 6.0, center.y - 2.0)], stroke);
    painter.line_segment([top, egui::pos2(center.x + 6.0, center.y - 2.0)], stroke);
}

fn draw_stop_icon(painter: &egui::Painter, rect: egui::Rect) {
    let center = rect.center();
    let stop_rect = egui::Rect::from_center_size(center, egui::vec2(9.0, 9.0));
    painter.rect_filled(stop_rect, 1.5, color(colors().text));
}
