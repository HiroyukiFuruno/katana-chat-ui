mod icons;

use self::icons::{PrimaryIcon, button_at, primary_button_at};
use crate::view::{EguiChatController, color, colors, layout, set_control_combo_visuals};
use eframe::egui;
use katana_chat_ui::{ChatUiSurface, ChatUiVendorControlSurface};

pub struct EguiComposerView;

const CONTROL_ROW_GAP: f32 = 10.0;
const ICON_BUTTON_SIZE: f32 = 38.0;
const USAGE_SIZE: f32 = 38.0;
const VENDOR_CONTROL_WIDTH: f32 = 220.0;

impl EguiComposerView {
    pub fn render(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let layout = layout();
        let colors = colors();
        egui::Frame::new()
            .fill(color(colors.assistant))
            .stroke(egui::Stroke::new(1.0, color(colors.border)))
            .corner_radius(layout.bubble_radius)
            .inner_margin(layout.composer_padding)
            .show(ui, |ui| Self::content(ui, surface, controller));
    }

    fn content(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let layout = layout();
        ui.set_width(ui.available_width().min(layout.chat_body_max_width));
        ui.add_sized(
            [ui.available_width(), layout.composer_input_height],
            egui::TextEdit::multiline(controller.composer_text_mut())
                .frame(egui::Frame::NONE)
                .hint_text(&surface.composer.placeholder),
        );
        ui.add_space(CONTROL_ROW_GAP);
        Self::controls(ui, surface, controller);
    }

    fn controls(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
    ) {
        let row_width = ui.available_width();
        let row_height = ICON_BUTTON_SIZE;
        let (row_rect, _) =
            ui.allocate_exact_size(egui::vec2(row_width, row_height), egui::Sense::hover());
        let attach_rect = egui::Rect::from_min_size(
            row_rect.left_top(),
            egui::vec2(ICON_BUTTON_SIZE, row_height),
        );
        if button_at(ui, attach_rect, "📎", &surface.composer.attach_label).clicked() {
            controller.attach();
        }
        let primary_rect = egui::Rect::from_min_size(
            egui::pos2(row_rect.right() - ICON_BUTTON_SIZE, row_rect.top()),
            egui::vec2(ICON_BUTTON_SIZE, row_height),
        );
        Self::primary_action(ui, surface, controller, primary_rect);
        let usage_rect = egui::Rect::from_min_size(
            egui::pos2(
                primary_rect.left() - CONTROL_ROW_GAP - USAGE_SIZE,
                row_rect.top(),
            ),
            egui::vec2(USAGE_SIZE, row_height),
        );
        usage_meter_at(ui, surface, usage_rect);
        Self::vendor_controls(ui, surface, controller, attach_rect, usage_rect, row_rect);
    }

    fn vendor_controls(
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
        let available_width =
            (usage_rect.left() - attach_rect.right() - (CONTROL_ROW_GAP * 2.0)).max(0.0);
        let total_gap = CONTROL_ROW_GAP * ((control_count - 1) as f32);
        let desired_width = (VENDOR_CONTROL_WIDTH * control_count as f32) + total_gap;
        let controls_width = desired_width.min(available_width);
        let min_left = attach_rect.right() + CONTROL_ROW_GAP;
        let max_left = usage_rect.left() - CONTROL_ROW_GAP - controls_width;
        let centered_left = row_rect.center().x - (controls_width / 2.0);
        let left = centered_left.clamp(min_left, max_left.max(min_left));
        let control_width = ((controls_width - total_gap) / control_count as f32).max(0.0);
        for (index, control) in surface.vendor_bar.controls.iter().enumerate() {
            let x = left + ((control_width + CONTROL_ROW_GAP) * index as f32);
            let rect = egui::Rect::from_min_size(
                egui::pos2(x, row_rect.top()),
                egui::vec2(control_width, row_rect.height()),
            );
            Self::vendor_control(ui, rect, control, controller);
        }
    }

    fn vendor_control(
        ui: &mut egui::Ui,
        rect: egui::Rect,
        control: &ChatUiVendorControlSurface,
        controller: &mut impl EguiChatController,
    ) {
        let mut selected = control.value.clone();
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            set_control_combo_visuals(ui);
            ui.add_enabled_ui(control.enabled, |ui| {
                egui::ComboBox::from_id_salt(format!("kcu-egui-control-{}", control.key))
                    .width(rect.width())
                    .selected_text(format!("{}: {}", control.label, control.value))
                    .show_ui(ui, |ui| {
                        for option in &control.options {
                            ui.selectable_value(&mut selected, option.clone(), option);
                        }
                    });
            });
        });
        if selected != control.value {
            controller.select_control(control.key.clone(), selected);
        }
    }

    fn primary_action(
        ui: &mut egui::Ui,
        surface: &ChatUiSurface,
        controller: &mut impl EguiChatController,
        rect: egui::Rect,
    ) {
        let icon = if surface.composer.stop_enabled {
            PrimaryIcon::Stop
        } else {
            PrimaryIcon::Send
        };
        let tooltip = if surface.composer.stop_enabled {
            &surface.composer.stop_label
        } else {
            &surface.composer.send_label
        };
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

fn usage_meter_at(ui: &mut egui::Ui, surface: &ChatUiSurface, rect: egui::Rect) {
    let response = ui.allocate_rect(rect, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();
    let radius = (USAGE_SIZE / 2.0) - 3.0;
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
