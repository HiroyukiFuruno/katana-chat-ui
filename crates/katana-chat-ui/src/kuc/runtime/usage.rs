use katana_ui_core::raster_host::{Canvas, UiTreeNodeHit};

const USAGE_NODE_ID: &str = "chat.composer.usage";
const RING_BACKGROUND: u32 = 0x606060;
const RING_ACTIVE: u32 = 0x4a90e2;

pub(super) fn paint(canvas: &mut Canvas, hits: &[UiTreeNodeHit], percent: u8) {
    let Some(hit) = hits
        .iter()
        .find(|hit| hit.node_id.as_str() == USAGE_NODE_ID)
    else {
        return;
    };
    let size = hit.rect.width.min(hit.rect.height);
    for y in 0..size {
        for x in 0..size {
            if let Some(color) = ring_pixel(x, y, size, percent) {
                canvas.fill_rect(
                    hit.rect.x.saturating_add(x),
                    hit.rect.y.saturating_add(y),
                    1,
                    1,
                    color,
                );
            }
        }
    }
}

fn ring_pixel(x: usize, y: usize, size: usize, percent: u8) -> Option<u32> {
    let radius = size as f32 / 2.0 - 1.0;
    let center = size as f32 / 2.0 - 0.5;
    let dx = x as f32 - center;
    let dy = y as f32 - center;
    let distance = (dx * dx + dy * dy).sqrt();
    if !(radius - 3.0..=radius).contains(&distance) {
        return None;
    }
    let angle = (dy.atan2(dx) + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
    let active = angle <= std::f32::consts::TAU * f32::from(percent) / 100.0;
    Some(if active { RING_ACTIVE } else { RING_BACKGROUND })
}
