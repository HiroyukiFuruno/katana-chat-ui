use katana_ui_core::raster_host::{Canvas, UiTreeCanvasRenderer, UiTreeRenderArea};
use katana_ui_core::render_model::{UiNode, UiNodeKind};
use katana_ui_core::theme::ThemeSnapshot;

#[test]
fn output_action_label_pixels_stay_inside_the_button() -> Result<(), Box<dyn std::error::Error>> {
    let surface = super::surface(crate::OutputStatus::Ready)?;
    let output = surface
        .output_handoff
        .outputs
        .first()
        .ok_or("output missing")?;
    use crate::HostActionKind::*;
    for kind in [
        Copy,
        OpenPreview,
        CreateFile,
        ApplyDiff,
        UndoChange,
        Approve,
        Reject,
    ] {
        let action = crate::HostActionIntent::new(output.id, kind);
        let button = super::super::action_node(&action, "test.output", 0)?;
        assert_label_contained(&button)?;
    }
    Ok(())
}

fn assert_label_contained(button: &UiNode) -> Result<(), Box<dyn std::error::Error>> {
    let renderer = UiTreeCanvasRenderer::new(ThemeSnapshot::light());
    let area = UiTreeRenderArea {
        x: 0,
        y: 0,
        width: 240,
        height: 100,
        scroll_y: 0.0,
    };
    let blank = UiNode::new(UiNodeKind::Button, "").common(button.props().common.clone());
    let mut labeled = Canvas::new(240, 100, 0);
    let mut unlabeled = Canvas::new(240, 100, 0);
    renderer.render(&mut labeled, button, area);
    renderer.render(&mut unlabeled, &blank, area);
    let (hits, _) = renderer.viewport_interaction_hit_rects(button, area);
    let rect = hits.first().ok_or("button hit missing")?.rect;
    assert_pixels_inside(&labeled, &unlabeled, rect);
    Ok(())
}

fn assert_pixels_inside(
    labeled: &Canvas,
    unlabeled: &Canvas,
    rect: katana_ui_core::raster_host::UiTreeHitRect,
) {
    let mut changed = 0;
    for (index, (left, right)) in labeled.pixels().iter().zip(unlabeled.pixels()).enumerate() {
        if left != right {
            changed += 1;
            let (x, y) = (index % 240, index / 240);
            assert!(
                x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height,
                "label pixel ({x}, {y}) outside button {rect:?}"
            );
        }
    }
    assert!(changed > 0, "button label was not rendered");
}
