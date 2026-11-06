use super::super::state::Rgba;
use super::text::COMPOSER_PADDING;
use katana_ui_core::raster_host::{Canvas, RgbaBlitRequest, RgbaSourceRect, UiTreeRenderArea};
use katana_ui_core::render_model::UiRect;
use katana_ui_core::text_raster::PlatformTextRaster;

pub(super) fn blit(
    canvas: &mut Canvas,
    bounds: UiRect,
    raster: &PlatformTextRaster,
    scale_factor: f32,
    scroll_y: i32,
) {
    let pixels = rgba_bytes(&raster.rgba_pixels);
    let width = logical_extent(raster.width, scale_factor).min(text_width(bounds));
    let height = logical_extent(raster.height, scale_factor).min(text_height(bounds));
    let source_y = physical_scroll_offset(scroll_y, scale_factor, raster.height);
    let source_height = physical_extent(height, scale_factor).min(raster.height - source_y);
    canvas.blit_rgba(RgbaBlitRequest {
        rgba: &pixels,
        width: raster.width as u32,
        height: raster.height as u32,
        source: RgbaSourceRect {
            x: 0.0,
            y: source_y as f32,
            width: raster.width as f32,
            height: source_height as f32,
        },
        area: UiTreeRenderArea {
            x: rect_x(bounds).saturating_add(COMPOSER_PADDING),
            y: rect_y(bounds).saturating_add(COMPOSER_PADDING),
            width,
            height,
            scroll_y: scroll_y.max(0) as f32,
        },
    });
}

fn physical_scroll_offset(scroll_y: i32, scale_factor: f32, raster_height: usize) -> usize {
    let offset = (scroll_y.max(0) as f32 * normalized_scale(scale_factor)).floor() as usize;
    offset.min(raster_height)
}

fn physical_extent(logical_extent: usize, scale_factor: f32) -> usize {
    (logical_extent as f32 * normalized_scale(scale_factor)).ceil() as usize
}

fn logical_extent(physical_extent: usize, scale_factor: f32) -> usize {
    (physical_extent as f32 / normalized_scale(scale_factor)).ceil() as usize
}

fn normalized_scale(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn text_width(bounds: UiRect) -> usize {
    bounds.width.saturating_sub((COMPOSER_PADDING * 2) as u32) as usize
}

fn text_height(bounds: UiRect) -> usize {
    bounds.height.saturating_sub((COMPOSER_PADDING * 2) as u32) as usize
}

fn rgba_bytes(pixels: &[Rgba]) -> Vec<u8> {
    pixels
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect()
}

fn rect_x(rect: UiRect) -> usize {
    usize::try_from(rect.x).unwrap_or(0)
}

fn rect_y(rect: UiRect) -> usize {
    usize::try_from(rect.y).unwrap_or(0)
}
