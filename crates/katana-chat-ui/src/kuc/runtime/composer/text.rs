use super::super::state::ChatUiRuntime;
use super::super::types::{ChatUiRuntimeError, ChatUiRuntimeViewport};
use katana_ui_core::atom::TextAreaCompositionState;
use katana_ui_core::raster_host::{Canvas, RgbaBlitRequest, RgbaSourceRect, UiTreeRenderArea};
use katana_ui_core::render_model::UiRect;
use katana_ui_core::text_raster::{PlatformTextRaster, PlatformTextRasterRequest};
use katana_ui_core::text_surface::{TextSurface, TextSurfaceLayout, TextSurfacePoint};

const COMPOSER_PADDING: usize = 8;

pub(super) fn layout(
    runtime: &mut ChatUiRuntime,
    bounds: UiRect,
    viewport: ChatUiRuntimeViewport,
) -> Result<TextSurfaceLayout, ChatUiRuntimeError> {
    let value = render_text(&runtime.composer, "");
    if value.is_empty() {
        return Ok(TextSurfaceLayout::new("chat.composer.input", bounds));
    }
    let raster = raster(runtime, &value, bounds, viewport)?;
    let origin = TextSurfacePoint::new(
        bounds.x.saturating_add(COMPOSER_PADDING as i32),
        bounds.y.saturating_add(COMPOSER_PADDING as i32),
    );
    let state = &runtime.composer.state().text_area;
    Ok(state.composition.as_ref().map_or_else(
        || raster.text_surface_layout("chat.composer.input", origin),
        |composition| composition_layout(&raster, state, composition, origin),
    ))
}

pub(super) fn paint(
    runtime: &mut ChatUiRuntime,
    canvas: &mut Canvas,
    bounds: UiRect,
    viewport: ChatUiRuntimeViewport,
    placeholder: &str,
) -> Result<(), ChatUiRuntimeError> {
    let display = render_text(&runtime.composer, placeholder);
    if display.is_empty() {
        return Ok(());
    }
    let raster = raster(runtime, &display, bounds, viewport)?;
    blit(
        canvas,
        bounds,
        &raster,
        viewport.scale_factor,
        runtime.composer.state().scroll_y,
    );
    Ok(())
}

fn composition_layout(
    raster: &PlatformTextRaster,
    state: &katana_ui_core::atom::TextAreaState,
    composition: &TextAreaCompositionState,
    origin: TextSurfacePoint,
) -> TextSurfaceLayout {
    raster.text_surface_layout_with_composition(
        "chat.composer.input",
        origin,
        state.selection.start,
        state.selection.end,
        &composition.preedit,
        composition.caret,
    )
}

fn render_text(surface: &TextSurface, placeholder: &str) -> String {
    let state = &surface.state().text_area;
    match state.composition.as_ref() {
        Some(composition) => composed_text(
            &state.value,
            state.selection.start,
            state.selection.end,
            composition,
        ),
        None if state.value.is_empty() => placeholder.to_string(),
        None => state.value.clone(),
    }
}

fn composed_text(
    value: &str,
    selection_start: usize,
    selection_end: usize,
    composition: &TextAreaCompositionState,
) -> String {
    let (start, end) = ordered_range(value, selection_start, selection_end);
    value.get(..start).zip(value.get(end..)).map_or_else(
        || value.to_string(),
        |(before, after)| format!("{before}{}{after}", composition.preedit),
    )
}

fn ordered_range(value: &str, start: usize, end: usize) -> (usize, usize) {
    let start = start.min(value.len());
    let end = end.min(value.len());
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn raster(
    runtime: &mut ChatUiRuntime,
    text: &str,
    bounds: UiRect,
    viewport: ChatUiRuntimeViewport,
) -> Result<PlatformTextRaster, ChatUiRuntimeError> {
    let mut request =
        PlatformTextRasterRequest::from_text(text, runtime.font.clone(), runtime.composer_text);
    request.max_width_px = Some(bounds.width.saturating_sub((COMPOSER_PADDING * 2) as u32) as f32);
    request.scale_factor = viewport.scale_factor;
    runtime.rasterizer.rasterize(&request).map_err(Into::into)
}

fn blit(
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

fn rgba_bytes(pixels: &[[u8; 4]]) -> Vec<u8> {
    pixels
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect()
}

fn rect_x(rect: UiRect) -> usize {
    usize::try_from(rect.x).map_or(0, |value| value)
}

fn rect_y(rect: UiRect) -> usize {
    usize::try_from(rect.y).map_or(0, |value| value)
}
