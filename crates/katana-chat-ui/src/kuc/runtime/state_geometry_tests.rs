use super::super::types::{ChatUiRuntimeFrame, ChatUiRuntimeViewport};
use super::ChatUiRuntime;
use crate::{ChatSession, ChatUiSurface};
use katana_ui_core::atom::Text;
use katana_ui_core::raster_host::{Canvas, UiTreeRenderArea};
use katana_ui_core::render_model::{UiNode, UiTextSpan, UiTextSpanStyle, UiTextWrapMode};
use katana_ui_core::theme::{ColorToken, ThemeSnapshot};

const WIDTH: usize = 640;
const HEIGHT: usize = 96;
const BACKGROUND: u32 = 0x151515;
const LINK_TARGET: &str = "https://example.test/arabic-middle-fragment";

struct StandaloneFixture {
    runtime: ChatUiRuntime,
    area: UiTreeRenderArea,
    underlined: UiNode,
    plain: UiNode,
    linked: UiNode,
}

#[test]
fn bidi_middle_fragment_underline_pixels_match_viewport_link_hit_geometry()
-> Result<(), Box<dyn std::error::Error>> {
    assert_standalone_fragment_geometry()
}

#[test]
fn production_markdown_arabic_link_underline_pixels_match_runtime_frame_hit()
-> Result<(), Box<dyn std::error::Error>> {
    assert_production_markdown_geometry()
}

fn assert_standalone_fragment_geometry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = standalone_fixture()?;
    let (min_x, min_y, max_x, max_y) = render_underline_delta(
        &fixture.runtime,
        fixture.area,
        &fixture.underlined,
        &fixture.plain,
    )?;
    let link_hit = viewport_link_hit(&fixture.runtime, &fixture.linked, fixture.area)?;

    assert_eq!(link_hit.rect.x, min_x);
    assert_eq!(
        link_hit.rect.width,
        max_x.saturating_sub(min_x).saturating_add(1)
    );
    assert_eq!(1, max_y.saturating_sub(min_y).saturating_add(1));
    assert!(link_hit.rect.height > 0);
    Ok(())
}

fn standalone_fixture() -> Result<StandaloneFixture, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let runtime = ChatUiRuntime::new(ThemeSnapshot::dark(), &surface)?;
    let area = UiTreeRenderArea {
        x: 0,
        y: 0,
        width: WIDTH,
        height: HEIGHT,
        scroll_y: 0.0,
    };
    Ok(StandaloneFixture {
        runtime,
        area,
        underlined: bidi_fragment_node(true, ""),
        plain: bidi_fragment_node(false, ""),
        linked: bidi_fragment_node(false, LINK_TARGET),
    })
}

fn render_underline_delta(
    runtime: &ChatUiRuntime,
    area: UiTreeRenderArea,
    underlined: &UiNode,
    plain: &UiNode,
) -> Result<(usize, usize, usize, usize), Box<dyn std::error::Error>> {
    let mut underlined_canvas = Canvas::new(WIDTH, HEIGHT, BACKGROUND);
    let mut plain_canvas = Canvas::new(WIDTH, HEIGHT, BACKGROUND);
    runtime
        .renderer
        .render(&mut underlined_canvas, underlined, area);
    runtime.renderer.render(&mut plain_canvas, plain, area);
    changed_pixel_bounds(plain_canvas.pixels(), underlined_canvas.pixels(), WIDTH)
        .ok_or_else(|| "underlining the middle fragment must change rendered pixels".into())
}

fn viewport_link_hit(
    runtime: &ChatUiRuntime,
    linked: &UiNode,
    area: UiTreeRenderArea,
) -> Result<katana_ui_core::raster_host::UiTreeHostActionHit, Box<dyn std::error::Error>> {
    runtime
        .renderer
        .viewport_interaction_hit_rects(linked, area)
        .0
        .into_iter()
        .find(|hit| hit.action.payload == LINK_TARGET)
        .ok_or_else(|| "middle-fragment viewport link hit was not produced".into())
}

fn assert_production_markdown_geometry() -> Result<(), Box<dyn std::error::Error>> {
    let default_theme = ThemeSnapshot::dark();
    let (linked_frame, plain_frame) = production_frame_pair_with_theme(default_theme)?;
    let default_pixel_bounds = production_frame_evidence(&linked_frame, &plain_frame);
    let default_hit = frame_link_hit(&linked_frame)?;
    let (default_min_x, default_min_y, default_max_x, default_max_y) = default_pixel_bounds
        .ok_or("default linked Markdown must render visible glyph and underline pixels")?;
    assert!(default_max_y > default_min_y);
    assert!(default_max_x >= default_min_x);
    assert_monochrome_underline_geometry(default_hit.rect)?;
    Ok(())
}

fn assert_monochrome_underline_geometry(
    default_hit: katana_ui_core::raster_host::UiTreeHitRect,
) -> Result<(), Box<dyn std::error::Error>> {
    let (linked_frame, plain_frame) =
        production_frame_pair_with_theme(theme_with_link_color_matching_text()?)?;
    let bounds = production_frame_evidence(&linked_frame, &plain_frame);
    let hit = frame_link_hit(&linked_frame)?;
    assert_eq!(default_hit, hit.rect);
    let (min_x, min_y, max_x, max_y) =
        bounds.ok_or("monochrome linked Markdown must retain an underline delta")?;
    assert_eq!(hit.rect.x, min_x);
    assert_eq!(
        hit.rect.width,
        max_x.saturating_sub(min_x).saturating_add(1)
    );
    assert_eq!(1, max_y.saturating_sub(min_y).saturating_add(1));
    Ok(())
}

fn production_frame_pair_with_theme(
    theme: ThemeSnapshot,
) -> Result<(ChatUiRuntimeFrame, ChatUiRuntimeFrame), Box<dyn std::error::Error>> {
    let linked_surface = markdown_surface("س[ل](https://example.test/arabic-middle-fragment)ا`x`")?;
    let plain_surface = markdown_surface("سلا`x`")?;
    let viewport = ChatUiRuntimeViewport::new(640, 480, 1.0);
    let mut linked_runtime = ChatUiRuntime::new(theme.clone(), &linked_surface)?;
    let mut plain_runtime = ChatUiRuntime::new(theme, &plain_surface)?;
    let linked_frame = linked_runtime.render(&linked_surface, viewport)?;
    let plain_frame = plain_runtime.render(&plain_surface, viewport)?;
    Ok((linked_frame, plain_frame))
}

fn theme_with_link_color_matching_text() -> Result<ThemeSnapshot, Box<dyn std::error::Error>> {
    let mut theme = ThemeSnapshot::dark();
    let text = theme
        .color("text")
        .ok_or("dark theme is missing required text color")?;
    // リンク色だけを本文色へ揃え、同じ実Markdown経路の差分を下線へ限定する。
    theme.colors.push(ColorToken {
        name: "link".to_owned(),
        rgba: text,
    });
    let link = theme
        .color("link")
        .ok_or("monochrome theme is missing required link color")?;
    assert_eq!(text, link);
    Ok(theme)
}

fn production_frame_evidence(
    linked_frame: &ChatUiRuntimeFrame,
    plain_frame: &ChatUiRuntimeFrame,
) -> Option<(usize, usize, usize, usize)> {
    changed_pixel_bounds(&plain_frame.rgba.pixels, &linked_frame.rgba.pixels, WIDTH)
}

fn frame_link_hit(
    frame: &ChatUiRuntimeFrame,
) -> Result<&katana_ui_core::raster_host::UiTreeHostActionHit, Box<dyn std::error::Error>> {
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload == LINK_TARGET)
        .ok_or_else(|| "ChatUiRuntime frame has no link hit".into())
}

fn markdown_surface(body: &str) -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.start_assistant_stream(body.to_owned())?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn bidi_fragment_node(underline: bool, link_target: &str) -> UiNode {
    Text::new("سلاx")
        .text_role("body")
        .wrap(UiTextWrapMode::Wrap)
        .text_spans(vec![
            UiTextSpan::plain("س"),
            UiTextSpan {
                text: "ل".to_owned(),
                style: UiTextSpanStyle {
                    underline,
                    ..UiTextSpanStyle::default()
                },
                link_target: link_target.to_owned(),
            },
            UiTextSpan::plain("ا"),
            UiTextSpan {
                text: "x".to_owned(),
                style: UiTextSpanStyle {
                    inline_code: true,
                    monospace: true,
                    ..UiTextSpanStyle::default()
                },
                link_target: String::new(),
            },
        ])
        .into()
}

fn changed_pixel_bounds(
    before: &[u32],
    after: &[u32],
    width: usize,
) -> Option<(usize, usize, usize, usize)> {
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for (index, (before_pixel, after_pixel)) in before.iter().zip(after).enumerate() {
        if before_pixel == after_pixel {
            continue;
        }
        let x = index % width;
        let y = index / width;
        bounds = Some(match bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    }
    bounds
}
