use super::super::{ChatUiRuntime, ChatUiRuntimeFrame, ChatUiRuntimeViewport};
use super::helpers::surface;

const LOGICAL_WIDTH: u32 = 640;
const LOGICAL_HEIGHT: u32 = 480;
const NORMAL_SCALE: f32 = 1.0;
const RETINA_SCALE: f32 = 2.0;
const COMPOSER_INSET: usize = 8;
const COMPOSER_BACKGROUND: u32 = 0xf3f3f3;

type PixelArea = (usize, usize, usize, usize);
type PixelBounds = (Option<usize>, Option<usize>, Option<usize>, Option<usize>);

#[test]
fn retina_text_raster_keeps_logical_layout_and_physical_density()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("retina text");
    let normal = render(&surface, NORMAL_SCALE)?;
    let retina = render(&surface, RETINA_SCALE)?;
    assert_eq!(normal.composer.layout, retina.composer.layout);
    assert_eq!(
        normal.composer.text_surface.selection,
        retina.composer.text_surface.selection
    );
    assert_eq!(
        normal.composer.text_surface.preedit,
        retina.composer.text_surface.preedit
    );
    assert_eq!(LOGICAL_WIDTH, normal.rgba.width);
    assert_eq!(LOGICAL_WIDTH * 2, retina.rgba.width);
    assert_scaled_text_extent(&normal, &retina)?;
    Ok(())
}

fn render(
    surface: &crate::ChatUiSurface,
    scale_factor: f32,
) -> Result<ChatUiRuntimeFrame, Box<dyn std::error::Error>> {
    let mut runtime = ChatUiRuntime::light(surface)?;
    runtime
        .render(
            surface,
            ChatUiRuntimeViewport::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, scale_factor),
        )
        .map_err(Into::into)
}

fn assert_scaled_text_extent(
    normal: &ChatUiRuntimeFrame,
    retina: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let normal_extent = text_extent(normal)?;
    let retina_extent = text_extent(retina)?;
    let retina_width = retina_extent.0 / 2;
    let retina_height = retina_extent.1 / 2;
    assert!(normal_extent.0.abs_diff(retina_width) <= 1);
    assert!(normal_extent.1.abs_diff(retina_height) <= 1);
    Ok(())
}

fn text_extent(frame: &ChatUiRuntimeFrame) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    text_extent_from_bounds(text_pixel_bounds(frame, composer_pixel_area(frame)))
}

fn composer_pixel_area(frame: &ChatUiRuntimeFrame) -> PixelArea {
    let scale = frame.viewport.scale_factor;
    let bounds = frame.composer.bounds;
    let left = (bounds.x as f32 * scale) as usize + COMPOSER_INSET;
    let top = (bounds.y as f32 * scale) as usize + COMPOSER_INSET;
    let width = (bounds.width as f32 * scale) as usize - COMPOSER_INSET * 2;
    let height = (bounds.height as f32 * scale) as usize - COMPOSER_INSET * 2;
    (left, top, width, height)
}

fn text_pixel_bounds(frame: &ChatUiRuntimeFrame, area: PixelArea) -> PixelBounds {
    let (left, top, width, height) = area;
    let pixels = frame.rgba.pixels.chunks_exact(frame.rgba.width as usize);
    let mut min_x = None;
    let mut min_y = None;
    let mut max_x = None;
    let mut max_y = None;
    for (y, row) in pixels.enumerate().skip(top).take(height) {
        for (x, pixel) in row.iter().enumerate().skip(left).take(width) {
            if *pixel != COMPOSER_BACKGROUND {
                min_x = Some(min_x.map_or(x, |value: usize| value.min(x)));
                min_y = Some(min_y.map_or(y, |value: usize| value.min(y)));
                max_x = Some(x);
                max_y = Some(y);
            }
        }
    }
    (min_x, min_y, max_x, max_y)
}

fn text_extent_from_bounds(bounds: PixelBounds) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) = bounds else {
        return Err(std::io::Error::other("missing text pixels").into());
    };
    Ok((
        u32::try_from(max_x.saturating_sub(min_x).saturating_add(1))?,
        u32::try_from(max_y.saturating_sub(min_y).saturating_add(1))?,
    ))
}
