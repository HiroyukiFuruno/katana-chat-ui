use super::super::state::ChatUiRuntime;
use super::super::types::{ChatUiRuntimeComposerFrame, ChatUiRuntimeError, ChatUiRuntimeViewport};
use super::text;
use katana_ui_core::raster_host::Canvas;
use katana_ui_core::render_model::UiRect;
use katana_ui_core::text_surface::TextSurfaceFrameRecord;

const SELECTION_ALPHA: u8 = 96;

pub(crate) fn frame(
    runtime: &mut ChatUiRuntime,
    canvas: &mut Canvas,
    bounds: UiRect,
    viewport: ChatUiRuntimeViewport,
    placeholder: &str,
    submit_enabled: bool,
) -> Result<ChatUiRuntimeComposerFrame, ChatUiRuntimeError> {
    synchronize_viewport(runtime, bounds);
    paint_background(canvas, bounds, runtime.composer_background);
    let layout = text::layout(runtime, bounds, viewport)?;
    runtime
        .composer
        .synchronize_scroll_bounds(layout.content_bounds, bounds);
    let text_surface = runtime.composer.frame_with_bounds(&layout, bounds, bounds);
    paint_content(
        runtime,
        canvas,
        bounds,
        viewport,
        placeholder,
        &text_surface,
    )?;
    Ok(ChatUiRuntimeComposerFrame {
        bounds,
        layout,
        text_surface,
        submit_enabled,
    })
}

fn synchronize_viewport(runtime: &mut ChatUiRuntime, bounds: UiRect) {
    let _ = runtime
        .composer
        .synchronize_measured_viewport_size(bounds.width, bounds.height);
}

fn paint_background(canvas: &mut Canvas, bounds: UiRect, color: u32) {
    canvas.fill_round_rect(
        rect_x(bounds),
        rect_y(bounds),
        bounds.width as usize,
        bounds.height as usize,
        8,
        color,
    );
}

fn paint_content(
    runtime: &mut ChatUiRuntime,
    canvas: &mut Canvas,
    bounds: UiRect,
    viewport: ChatUiRuntimeViewport,
    placeholder: &str,
    frame: &TextSurfaceFrameRecord,
) -> Result<(), ChatUiRuntimeError> {
    paint_selection(canvas, frame, runtime.composer_selection);
    text::paint(runtime, canvas, bounds, viewport, placeholder)?;
    paint_preedit_underline(canvas, frame, runtime.composer_text);
    paint_caret(canvas, frame, runtime.composer_text);
    Ok(())
}

fn paint_selection(canvas: &mut Canvas, frame: &TextSurfaceFrameRecord, color: u32) {
    for rect in &frame.selection.rects {
        canvas.blend_rect(
            rect_x(*rect),
            rect_y(*rect),
            rect.width as usize,
            rect.height as usize,
            color,
            SELECTION_ALPHA,
        );
    }
}

fn paint_caret(canvas: &mut Canvas, frame: &TextSurfaceFrameRecord, color: [u8; 4]) {
    let caret = frame.selection.caret;
    canvas.fill_rect(
        rect_x(caret),
        rect_y(caret),
        caret.width.max(1) as usize,
        caret.height.max(1) as usize,
        packed(color),
    );
}

fn paint_preedit_underline(canvas: &mut Canvas, frame: &TextSurfaceFrameRecord, color: [u8; 4]) {
    let Some(preedit) = &frame.preedit else {
        return;
    };
    for rect in &preedit.rects {
        canvas.fill_rect(
            rect_x(*rect),
            rect_y(*rect).saturating_add(rect.height.saturating_sub(1) as usize),
            rect.width.max(1) as usize,
            1,
            packed(color),
        );
    }
}

fn rect_x(rect: UiRect) -> usize {
    usize::try_from(rect.x).unwrap_or(0)
}

fn rect_y(rect: UiRect) -> usize {
    usize::try_from(rect.y).unwrap_or(0)
}

fn packed(color: [u8; 4]) -> u32 {
    (u32::from(color[0]) << 16) | (u32::from(color[1]) << 8) | u32::from(color[2])
}
