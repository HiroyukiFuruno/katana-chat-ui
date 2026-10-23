use super::ChatUiRuntime;
use crate::{ChatSession, ChatUiSurface};
use katana_ui_core::atom::Text;
use katana_ui_core::raster_host::{Canvas, UiTreeHostActionHit, UiTreeRenderArea};
use katana_ui_core::render_model::{UiNode, UiTextSpan, UiTextSpanStyle, UiTextWrapMode};
use katana_ui_core::theme::ThemeSnapshot;

#[test]
fn code_wrap_drawing_hit_lines_and_copy_share_the_same_layout()
-> Result<(), Box<dyn std::error::Error>> {
    let session = ChatSession::new();
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let runtime = ChatUiRuntime::new(ThemeSnapshot::dark(), &surface)?;
    let text = "a = b; a = b; a = b; a = b; a = b; a = b;";
    let root = wrapped_code_node(text);
    for width in [24, 48, 120] {
        assert_wrap_layout(&runtime, &root, text, width);
    }
    Ok(())
}

fn wrapped_code_node(text: &str) -> UiNode {
    Text::new(text)
        .text_role("markdown-paragraph")
        .wrap(UiTextWrapMode::Wrap)
        .text_spans(vec![UiTextSpan {
            text: text.to_owned(),
            style: UiTextSpanStyle {
                monospace: true,
                inline_code: true,
                ..UiTextSpanStyle::default()
            },
            link_target: "https://example.test/wrapped-code".to_owned(),
        }])
        .into()
}

fn assert_wrap_layout(runtime: &ChatUiRuntime, root: &UiNode, text: &str, width: usize) {
    let area = UiTreeRenderArea {
        x: 0,
        y: 0,
        width,
        height: 1_200,
        scroll_y: 0.0,
    };
    let hits = runtime.renderer.viewport_host_action_hit_rects(root, area);
    let mut canvas = Canvas::new(width, area.height, 0);
    runtime.renderer.render(&mut canvas, root, area);
    assert!(hits.len() > 1, "code must wrap at width {width}");
    assert_hits_match_drawn_lines(&hits, &canvas, width);
    assert_eq!(
        Some(text.to_owned()),
        canvas.copy_text_in_selection(Some((0, 0)), Some((width, area.height))),
        "wrapping must preserve copied source at width {width}"
    );
}

fn assert_hits_match_drawn_lines(hits: &[UiTreeHostActionHit], canvas: &Canvas, width: usize) {
    for hit in hits {
        assert!(hit.rect.x + hit.rect.width <= width);
        assert!(
            canvas
                .text_runs()
                .iter()
                .any(|run| run.y() == hit.rect.y && run.x() <= hit.rect.x),
            "hit must share the drawn line at width {width}"
        );
    }
}
