use crate::view::{color, colors, layout};
use eframe::egui;
use katana_chat_ui::ChatUiSurface;

pub(super) fn usage_meter_at(ui: &mut egui::Ui, surface: &ChatUiSurface, rect: egui::Rect) {
    let response = ui.allocate_rect(rect, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = (super::layout::USAGE_SIZE / 2.0) - 3.0;
    painter.circle_stroke(
        center,
        radius,
        egui::Stroke::new(3.0, color(colors().border)),
    );
    painter.circle_stroke(center, radius, egui::Stroke::new(3.0, color(colors().text)));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        format!("{}%", surface.usage.context_percentage),
        egui::FontId::proportional(layout().font_meta),
        color(colors().text),
    );
    response.on_hover_text(usage_detail(surface));
}

fn usage_detail(surface: &ChatUiSurface) -> String {
    if surface.usage.max_tokens == 0 {
        return "コンテキスト画面:\n使用率 --%\n使用済み token: 不明\n\n提供元（provider）ごとに上限と管理方法が変わります"
            .to_string();
    }
    format!(
        "コンテキスト画面:\n使用率 {}%\n使用済み token: {} / {}\n\n提供元（provider）ごとに上限と管理方法が変わります\nstatus: {}\naccount: {}",
        surface.usage.context_percentage,
        surface.usage.used_tokens,
        surface.usage.max_tokens,
        surface.usage.context_status,
        surface.usage.account_label
    )
}
