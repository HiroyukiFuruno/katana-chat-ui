use katana_ui_core::text_surface::TextSurfacePoint;

pub(super) fn point(x: f32, y: f32) -> TextSurfacePoint {
    TextSurfacePoint::new(x.round() as i32, y.round() as i32)
}
