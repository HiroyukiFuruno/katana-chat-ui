use std::sync::Arc;

use ::image::{Frame, RgbaImage};
use gpui::RenderImage;
use katana_chat_ui::kuc::ChatUiRuntimeRgba;

use super::GpuiKucViewError;

pub(super) fn render_image(rgba: &ChatUiRuntimeRgba) -> Result<Arc<RenderImage>, GpuiKucViewError> {
    let bytes = bgra_bytes(&rgba.pixels);
    let image =
        RgbaImage::from_raw(rgba.width, rgba.height, bytes).ok_or(GpuiKucViewError::InvalidRgba)?;
    Ok(Arc::new(RenderImage::new(vec![Frame::new(image)])))
}

fn bgra_bytes(pixels: &[u32]) -> Vec<u8> {
    pixels
        .iter()
        .flat_map(|pixel| (pixel | 0xff00_0000).to_le_bytes())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::render_image;
    use katana_chat_ui::kuc::ChatUiRuntimeRgba;

    #[test]
    fn gpui_image_keeps_kuc_rgb_opaque_in_bgra_order() -> Result<(), Box<dyn std::error::Error>> {
        let image = render_image(&ChatUiRuntimeRgba {
            width: 2,
            height: 1,
            pixels: vec![0x00123456, 0x00ffffff],
        })?;
        assert_eq!(
            image.as_bytes(0),
            Some([0x56, 0x34, 0x12, 0xff, 0xff, 0xff, 0xff, 0xff].as_slice())
        );
        Ok(())
    }
}
