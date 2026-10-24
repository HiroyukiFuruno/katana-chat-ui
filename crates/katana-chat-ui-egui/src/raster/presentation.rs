use eframe::egui;
use katana_chat_ui::kuc::ChatUiRuntimeFrame;

pub(super) fn paint(
    ui: &egui::Ui,
    rect: egui::Rect,
    frame: &ChatUiRuntimeFrame,
    texture: &mut Option<egui::TextureHandle>,
) {
    let pixels = frame
        .rgba
        .pixels
        .iter()
        .map(|pixel| color(*pixel))
        .collect();
    let image = egui::ColorImage::new(
        [frame.rgba.width as usize, frame.rgba.height as usize],
        pixels,
    );
    let texture = match texture {
        Some(texture) => {
            texture.set(image, egui::TextureOptions::LINEAR);
            texture
        }
        None => texture.insert(ui.ctx().load_texture(
            "katana-chat-ui.kuc",
            image,
            egui::TextureOptions::LINEAR,
        )),
    };
    let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
    ui.painter()
        .image(texture.id(), rect, uv, egui::Color32::WHITE);
}

pub(super) fn ime(ui: &egui::Ui, rect: egui::Rect, frame: &ChatUiRuntimeFrame) {
    let input = frame.composer.bounds;
    let selection = &frame.composer.text_surface;
    let caret = selection
        .preedit
        .as_ref()
        .map_or(selection.selection.caret, |it| it.caret);
    let input_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(input.x as f32, input.y as f32),
        egui::vec2(input.width as f32, input.height as f32),
    );
    let cursor_rect = if frame.composer.layout.text().is_empty() {
        // 空のKUC layoutはglyph caretを持たないため、native IMEには入力領域内のanchorを渡す。
        egui::Rect::from_min_size(input_rect.min, egui::Vec2::splat(1.0))
    } else {
        egui::Rect::from_min_size(
            rect.min + egui::vec2(caret.x as f32, caret.y as f32),
            egui::vec2(caret.width.max(1) as f32, caret.height.max(1) as f32),
        )
    };
    ui.ctx().output_mut(|output| {
        output.ime = Some(egui::output::IMEOutput {
            purpose: egui::IMEPurpose::Normal,
            should_interrupt_composition: false,
            rect: input_rect,
            cursor_rect,
        });
    });
}

fn color(pixel: u32) -> egui::Color32 {
    egui::Color32::from_rgb((pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8)
}

#[cfg(test)]
mod tests {
    use super::color;
    use eframe::egui;

    #[test]
    fn kuc_rgb_pixels_remain_opaque_in_egui_texture() {
        assert_eq!(color(0x12_ab_34), egui::Color32::from_rgb(0x12, 0xab, 0x34));
        assert_eq!(color(0), egui::Color32::BLACK);
    }
}
