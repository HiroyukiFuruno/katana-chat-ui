use super::{ChatUiRuntime, chat_code_text_raster_config};
use crate::{ChatSession, ChatUiSurface};
use katana_ui_core::atom::Text;
use katana_ui_core::raster_host::{
    Canvas, SelectableTextRun, UiTreeCanvasRenderer, UiTreeHitRect, UiTreeRenderArea,
};
use katana_ui_core::render_model::{UiNode, UiTextSpan, UiTextSpanStyle};
use katana_ui_core::text_raster::{PlatformTextFaceSelection, PlatformTextRasterConfig};
use katana_ui_core::theme::ThemeSnapshot;

const WIDTH: usize = 640;
const HEIGHT: usize = 160;
const LINK: &str = "https://example.test/bidi-code";
const SELECTION_COLOR: u32 = 0x00ff00;

#[test]
fn code_raster_preserves_the_existing_system_face_pixels() -> Result<(), Box<dyn std::error::Error>>
{
    let session = ChatSession::new();
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let runtime = ChatUiRuntime::new(ThemeSnapshot::dark(), &surface)?;
    let text = "fn main() {}";
    let body: UiNode = Text::new(text).text_role("body").into();
    let code: UiNode = Text::new(text)
        .text_role("body")
        .text_spans(vec![UiTextSpan {
            text: text.to_owned(),
            style: UiTextSpanStyle {
                monospace: true,
                ..UiTextSpanStyle::default()
            },
            link_target: String::new(),
        }])
        .into();
    let mut body_canvas = Canvas::new(WIDTH, HEIGHT, 0);
    let mut code_canvas = Canvas::new(WIDTH, HEIGHT, 0);
    runtime
        .renderer
        .render(&mut body_canvas, &body, render_area());
    runtime
        .renderer
        .render(&mut code_canvas, &code, render_area());
    assert_eq!(body_canvas.pixels(), code_canvas.pixels());
    assert!(!code_canvas.text_runs().is_empty());
    Ok(())
}

#[test]
fn runtime_code_raster_keeps_body_system_rendering_and_monospace_model_semantics()
-> Result<(), Box<dyn std::error::Error>> {
    let theme = ThemeSnapshot::dark();
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.start_assistant_stream("body text with `inline code`")?;
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let mut runtime = ChatUiRuntime::new(theme.clone(), &surface)?;
    let frame = runtime.render(
        &surface,
        super::super::types::ChatUiRuntimeViewport::new(640, 480, 1.0),
    )?;
    assert!(tree_has_monospace_inline_code(frame.tree.root()));

    assert_body_renderer_pixels_match(&mut runtime, theme)?;
    Ok(())
}

fn assert_body_renderer_pixels_match(
    runtime: &mut ChatUiRuntime,
    theme: ThemeSnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    let body: UiNode = Text::new("body face stays on the system renderer")
        .text_role("body")
        .into();
    let baseline_renderer = UiTreeCanvasRenderer::with_text_raster_config(
        theme,
        PlatformTextRasterConfig::default(),
        PlatformTextFaceSelection::System,
    );
    let control_renderer = UiTreeCanvasRenderer::with_text_raster_config(
        ThemeSnapshot::dark(),
        PlatformTextRasterConfig::default(),
        PlatformTextFaceSelection::System,
    )
    .with_code_text_raster_config(
        chat_code_text_raster_config(),
        PlatformTextFaceSelection::CandidateChain,
    );
    let runtime_canvas = render_body(&runtime.renderer, &body);
    let baseline_canvas = render_body(&baseline_renderer, &body);
    let control_canvas = render_body(&control_renderer, &body);
    assert_canvases_match(&baseline_canvas, &runtime_canvas);
    assert_canvases_match(&baseline_canvas, &control_canvas);
    Ok(())
}

fn render_body(renderer: &UiTreeCanvasRenderer, body: &UiNode) -> Canvas {
    let mut canvas = Canvas::new(WIDTH, HEIGHT, 0);
    renderer.render(&mut canvas, body, render_area());
    canvas
}

fn assert_canvases_match(expected: &Canvas, actual: &Canvas) {
    assert_eq!(expected.pixels(), actual.pixels());
    assert_eq!(expected.text_runs(), actual.text_runs());
}

#[test]
fn runtime_bidi_inline_code_selection_link_hit_and_copy_keep_each_separator()
-> Result<(), Box<dyn std::error::Error>> {
    let theme = ThemeSnapshot::dark();
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let runtime = ChatUiRuntime::new(theme, &surface)?;
    let area = render_area();

    for separator in ["\r\n", "\r", "\n", "\u{2028}", "\u{2029}"] {
        assert_separator_contract(&runtime, area, separator)?;
    }

    Ok(())
}

fn assert_separator_contract(
    runtime: &ChatUiRuntime,
    area: UiTreeRenderArea,
    separator: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = bidi_inline_code_node(separator);
    let link_hit = runtime
        .renderer
        .document_host_action_hit_rects(&root, area)
        .into_iter()
        .find(|hit| hit.action.payload == LINK)
        .ok_or("inline-code link hit was not produced")?;
    let mut canvas = Canvas::new(WIDTH, HEIGHT, 0);
    runtime.renderer.render(&mut canvas, &root, area);
    let code_run = canvas
        .text_runs()
        .iter()
        .find(|run| run.text().contains("xyz"))
        .ok_or("inline-code raster run was not produced")?;
    assert_hit_matches_run(link_hit.rect, code_run);
    assert_eq!(
        Some(format!("אבג{separator}xyz")),
        canvas.copy_text_in_selection(Some((0, 0)), Some((code_run.right(), code_run.bottom()))),
        "selection copy must preserve the source separator {separator:?}"
    );
    let selection_bounds = code_run_center_selection(code_run);
    assert_code_selection_copy(&canvas, selection_bounds, separator);
    assert_selection_pixels_stay_inside_link(&mut canvas, selection_bounds, link_hit.rect)
}

fn code_run_center_selection(run: &SelectableTextRun) -> ((usize, usize), (usize, usize)) {
    // 上下端は隣接行と共有するため、同じ code run の中央 y を選択点に使う。
    let center_y = run.y() + (run.bottom() - run.y()) / 2;
    ((run.x(), center_y), (run.right(), center_y))
}

fn assert_code_selection_copy(
    canvas: &Canvas,
    selection: ((usize, usize), (usize, usize)),
    separator: &str,
) {
    assert_eq!(
        Some("xyz".to_owned()),
        canvas.copy_text_in_selection(Some(selection.0), Some(selection.1)),
        "code-only selection must copy only the linked code span for {separator:?}"
    );
}

fn bidi_inline_code_node(separator: &str) -> UiNode {
    let (leading, code) = if separator == "\r\n" {
        ("אבג\r".to_owned(), "\nxyz".to_owned())
    } else {
        (format!("אבג{separator}"), "xyz".to_owned())
    };
    Text::new(format!("אבג{separator}xyz"))
        .text_role("body")
        .text_spans(vec![
            UiTextSpan::plain(leading),
            UiTextSpan {
                text: code,
                style: UiTextSpanStyle {
                    inline_code: true,
                    monospace: true,
                    underline: true,
                    ..UiTextSpanStyle::default()
                },
                link_target: LINK.to_owned(),
            },
        ])
        .into()
}

fn assert_hit_matches_run(rect: UiTreeHitRect, run: &SelectableTextRun) {
    assert!(rect.width > 0 && rect.height > 0);
    assert_eq!(run.x(), rect.x);
    assert_eq!(run.y(), rect.y);
    assert!(run.width().abs_diff(rect.width) <= 1);
    assert!(run.height().abs_diff(rect.height) <= 1);
}

fn assert_selection_pixels_stay_inside_link(
    canvas: &mut Canvas,
    selection_bounds: ((usize, usize), (usize, usize)),
    link_rect: UiTreeHitRect,
) -> Result<(), Box<dyn std::error::Error>> {
    let before_selection = canvas.pixels().to_vec();
    assert!(canvas.draw_text_selection_highlight(
        Some(selection_bounds.0),
        Some(selection_bounds.1),
        SELECTION_COLOR,
    ));
    let changed_inside_link = changed_pixels_are_inside_link(canvas, &before_selection, link_rect);
    assert!(
        changed_inside_link,
        "selection paint must cover the linked code span"
    );
    Ok(())
}

fn changed_pixels_are_inside_link(canvas: &Canvas, before: &[u32], rect: UiTreeHitRect) -> bool {
    let mut changed_inside_link = false;
    let mut changed_count = 0;
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let index = y * canvas.width() + x;
            if before[index] == canvas.pixels()[index] {
                continue;
            }
            changed_count += 1;
            assert!(
                pixel_near_link(x, y, rect),
                "selection paint escaped link geometry at ({x},{y}); hit={rect:?}"
            );
            changed_inside_link |= pixel_inside_link(x, y, rect);
        }
    }
    assert!(
        changed_count > 0 && changed_inside_link,
        "selection paint must change at least one pixel inside link geometry; hit={rect:?}"
    );
    changed_inside_link
}

fn pixel_near_link(x: usize, y: usize, rect: UiTreeHitRect) -> bool {
    x.saturating_add(1) >= rect.x
        && x <= rect.x.saturating_add(rect.width)
        && y.saturating_add(1) >= rect.y
        && y <= rect.y.saturating_add(rect.height)
}

fn pixel_inside_link(x: usize, y: usize, rect: UiTreeHitRect) -> bool {
    x >= rect.x
        && x < rect.x.saturating_add(rect.width)
        && y >= rect.y
        && y < rect.y.saturating_add(rect.height)
}

fn render_area() -> UiTreeRenderArea {
    UiTreeRenderArea {
        x: 0,
        y: 0,
        width: WIDTH,
        height: HEIGHT,
        scroll_y: 0.0,
    }
}

fn tree_has_monospace_inline_code(node: &UiNode) -> bool {
    node.props()
        .text
        .spans
        .iter()
        .any(|span| span.style.inline_code && span.style.monospace)
        || node.children().iter().any(tree_has_monospace_inline_code)
}
