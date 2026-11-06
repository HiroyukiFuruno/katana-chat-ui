use super::super::state::ChatUiRuntime;
use super::super::types::{ChatUiRuntimeError, ChatUiRuntimeViewport};
use super::pixels::blit;
use katana_ui_core::atom::TextAreaCompositionState;
use katana_ui_core::raster_host::Canvas;
use katana_ui_core::render_model::UiRect;
use katana_ui_core::text_raster::{PlatformTextRaster, PlatformTextRasterRequest};
use katana_ui_core::text_selection::UiTextSelectionRange;
use katana_ui_core::text_surface::{TextSurface, TextSurfaceLayout, TextSurfacePoint};

pub(super) const COMPOSER_PADDING: usize = 8;

pub(super) fn layout(
    runtime: &mut ChatUiRuntime,
    bounds: UiRect,
    viewport: ChatUiRuntimeViewport,
) -> Result<TextSurfaceLayout, ChatUiRuntimeError> {
    let value = render_text(&runtime.composer, "");
    let origin = TextSurfacePoint::new(
        bounds.x.saturating_add(COMPOSER_PADDING as i32),
        bounds.y.saturating_add(COMPOSER_PADDING as i32),
    );
    if value.is_empty() {
        /* WHY: 空文字には測定対象がないため、値へ追加せずKUCの空白行からcaretの高さを得る。 */
        let measured = raster(runtime, " ", bounds, viewport)?
            .text_surface_layout("chat.composer.input", origin);
        let caret = measured.caret_rect(UiTextSelectionRange::caret(0));
        return Ok(TextSurfaceLayout::from_grapheme_boxes(
            "chat.composer.input",
            caret,
            "",
            Vec::new(),
        ));
    }
    let raster = raster(runtime, &value, bounds, viewport)?;
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
