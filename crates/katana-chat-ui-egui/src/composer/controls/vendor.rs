use crate::view::{EguiChatController, set_control_combo_visuals};
use eframe::egui;
use katana_chat_ui::ChatUiVendorControlSurface;

pub(super) fn render_vendor_control(
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
