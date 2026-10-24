use gpui::{Bounds, Pixels, point, px};

use super::super::GpuiKucView;

#[derive(Clone, Copy)]
pub(super) struct OffsetSnapshot {
    pub(super) paint_bounds: Bounds<Pixels>,
    pub(super) composer_point: gpui::Point<Pixels>,
    pub(super) expected_index: usize,
    pub(super) ime_bounds: Bounds<Pixels>,
}

impl OffsetSnapshot {
    pub(super) fn from_view(view: &GpuiKucView) -> Result<Self, String> {
        let frame = view.frame.as_ref().ok_or("missing KUC frame")?;
        let paint_bounds = view.paint_bounds.ok_or("missing GPUI paint bounds")?;
        let composer = frame.composer.bounds;
        let local_x = composer.x.saturating_add(1);
        let local_y = composer.y.saturating_add(1);
        Ok(Self {
            paint_bounds,
            composer_point: window_point(paint_bounds, local_x, local_y),
            expected_index: expected_index(view, local_x, local_y)?,
            ime_bounds: ime_bounds(view, paint_bounds)?,
        })
    }
}

fn expected_index(view: &GpuiKucView, x: i32, y: i32) -> Result<usize, String> {
    let frame = view.frame.as_ref().ok_or("missing KUC frame")?;
    let byte = frame.composer.layout.hit_test(x, y).caret_position();
    view.draft
        .get(..byte)
        .map(|prefix| prefix.encode_utf16().count())
        .ok_or_else(|| "KUC caret was outside the draft".into())
}

fn ime_bounds(view: &GpuiKucView, paint_bounds: Bounds<Pixels>) -> Result<Bounds<Pixels>, String> {
    let frame = view.frame.as_ref().ok_or("missing KUC frame")?;
    let ime = frame
        .composer
        .layout
        .bounds_for_byte_range(0, 0)
        .ok_or("KUC composer has no IME caret bounds")?;
    Ok(Bounds::from_corners(
        window_point(paint_bounds, ime.x, ime.y),
        window_point(
            paint_bounds,
            ime.x + ime.width as i32,
            ime.y + ime.height as i32,
        ),
    ))
}

fn window_point(bounds: Bounds<Pixels>, x: i32, y: i32) -> gpui::Point<Pixels> {
    point(bounds.left() + px(x as f32), bounds.top() + px(y as f32))
}
