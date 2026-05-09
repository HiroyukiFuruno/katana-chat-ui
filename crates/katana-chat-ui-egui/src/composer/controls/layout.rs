use super::super::{CONTROL_ROW_GAP as COMPOSER_ROW_GAP, ICON_BUTTON_SIZE};
use eframe::egui;

pub(super) const CONTROL_ROW_GAP: f32 = COMPOSER_ROW_GAP;
pub(super) const USAGE_SIZE: f32 = 38.0;
const VENDOR_CONTROL_WIDTH: f32 = 220.0;
const COMPACT_CONTROL_MIN_WIDTH: f32 = 150.0;

pub(super) type ControlRects = (egui::Rect, egui::Rect, egui::Rect, egui::Rect);

pub(super) fn use_compact_controls(row_width: f32, control_count: usize) -> bool {
    control_count > 0 && available_vendor_width(row_width) < minimum_vendor_width(control_count)
}

pub(super) fn compact_control_width(row_width: f32, control_count: usize) -> f32 {
    let gaps = CONTROL_ROW_GAP * control_count.saturating_sub(1) as f32;
    ((row_width - gaps) / control_count as f32).max(0.0)
}

pub(super) fn wide_controls_width(
    attach_rect: egui::Rect,
    usage_rect: egui::Rect,
    control_count: usize,
) -> f32 {
    let available_width =
        (usage_rect.left() - attach_rect.right() - (CONTROL_ROW_GAP * 2.0)).max(0.0);
    let total_gap = CONTROL_ROW_GAP * ((control_count - 1) as f32);
    let desired_width = (VENDOR_CONTROL_WIDTH * control_count as f32) + total_gap;
    desired_width.min(available_width)
}

pub(super) fn wide_controls_left(
    row_rect: egui::Rect,
    attach_rect: egui::Rect,
    usage_rect: egui::Rect,
    controls_width: f32,
) -> f32 {
    let min_left = attach_rect.right() + CONTROL_ROW_GAP;
    let max_left = usage_rect.left() - CONTROL_ROW_GAP - controls_width;
    let centered_left = row_rect.center().x - (controls_width / 2.0);
    centered_left.clamp(min_left, max_left.max(min_left))
}

pub(super) fn row_control_width(controls_width: f32, control_count: usize) -> f32 {
    let total_gap = CONTROL_ROW_GAP * ((control_count - 1) as f32);
    ((controls_width - total_gap) / control_count as f32).max(0.0)
}

fn available_vendor_width(row_width: f32) -> f32 {
    row_width - ((ICON_BUTTON_SIZE * 3.0) + (CONTROL_ROW_GAP * 4.0))
}

fn minimum_vendor_width(control_count: usize) -> f32 {
    (COMPACT_CONTROL_MIN_WIDTH * control_count as f32)
        + (CONTROL_ROW_GAP * control_count.saturating_sub(1) as f32)
}
