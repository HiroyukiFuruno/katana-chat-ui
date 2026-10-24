use super::super::super::types::ChatUiRuntimeFrame;

pub(super) fn inside_composer(frame: &ChatUiRuntimeFrame, x: f32, y: f32) -> bool {
    let bounds = frame.composer.bounds;
    x >= bounds.x as f32
        && y >= bounds.y as f32
        && x < bounds.x.saturating_add_unsigned(bounds.width) as f32
        && y < bounds.y.saturating_add_unsigned(bounds.height) as f32
}
