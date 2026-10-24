use super::super::GpuiKucView;
use gpui::{Entity, EntityInputHandler, TestAppContext, px, size};
use katana_chat_ui::{ChatSession, ChatUiSurface, kuc::ChatUiRuntimeInput};

#[gpui::test]
fn marked_text_exposes_a_virtual_document_and_commits_its_base_range(cx: &mut TestAppContext) {
    assert_eq!(run_marked_text_adapter(cx), Ok(()));
}

#[gpui::test]
fn empty_draft_marks_japanese_text_in_the_virtual_document(cx: &mut TestAppContext) {
    assert_eq!(run_empty_draft_adapter(cx), Ok(()));
}

#[gpui::test]
fn empty_preedit_clears_the_marked_range_without_changing_the_draft(cx: &mut TestAppContext) {
    assert_eq!(run_empty_preedit_clear(cx), Ok(()));
}

fn run_empty_preedit_clear(cx: &mut TestAppContext) -> Result<(), String> {
    let (view, cx) = view_for("prefix", cx)?;
    focus_composer(&view, cx)?;
    replace_preedit(&view, cx, Some(6..6), "日本", Some(1..1));
    draw(cx);
    assert!(virtual_snapshot(&view, cx, 8).1.is_some());

    replace_preedit(&view, cx, Some(6..8), "", Some(0..0));
    draw(cx);
    let (text, marked, _) = virtual_snapshot(&view, cx, 6);
    assert_eq!(text.as_deref(), Some("prefix"));
    assert!(marked.is_none());
    Ok(())
}

fn run_marked_text_adapter(cx: &mut TestAppContext) -> Result<(), String> {
    let (view, cx) = view_for("abc", cx)?;
    focus_composer(&view, cx)?;
    replace_preedit(&view, cx, Some(0..0), "日本", Some(1..1));
    draw(cx);
    assert_virtual_document(&view, cx, "日本abc", 0..2, 1..1)?;
    assert_preedit_caret(&view, cx)?;
    replace_preedit(&view, cx, Some(0..2), "語", Some(1..1));
    assert_virtual_document(&view, cx, "語abc", 0..1, 1..1)?;
    commit_marked_text(&view, cx, Some(0..1), "字");
    assert_committed_draft(&view, cx, "字abc")
}

fn run_empty_draft_adapter(cx: &mut TestAppContext) -> Result<(), String> {
    let (view, cx) = view_for("", cx)?;
    focus_composer(&view, cx)?;
    replace_preedit(&view, cx, Some(0..0), "日本", Some(1..1));
    draw(cx);
    assert_virtual_document(&view, cx, "日本", 0..2, 1..1)
}

fn view_for<'a>(
    draft: &str,
    cx: &'a mut TestAppContext,
) -> Result<(Entity<GpuiKucView>, &'a mut gpui::VisualTestContext), String> {
    let surface = surface(draft);
    let runtime = GpuiKucView::prepare(&surface).map_err(|error| error.to_string())?;
    let (view, cx) = cx.add_window_view(|_, view_cx| {
        GpuiKucView::from_runtime(runtime, surface, |_, _| None, view_cx)
    });
    draw(cx);
    Ok((view, cx))
}

fn focus_composer(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
) -> Result<(), String> {
    cx.update(|window, app| {
        view.update(app, |view, view_cx| {
            let frame = view.frame.as_ref().ok_or("KUC frame was not drawn")?;
            let bounds = frame.composer.bounds;
            view.dispatch_input(
                ChatUiRuntimeInput::PointerPress {
                    x: bounds.x as f32 + 1.0,
                    y: bounds.y as f32 + 1.0,
                    extend_selection: false,
                },
                window,
                view_cx,
            );
            Ok(())
        })
    })
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.simulate_resize(size(px(320.0), px(240.0)));
    cx.update(|window, app| {
        let _ = window.draw(app);
    });
}

fn replace_preedit(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    range: Option<std::ops::Range<usize>>,
    text: &str,
    selected: Option<std::ops::Range<usize>>,
) {
    cx.update(|window, app| {
        view.update(app, |view, view_cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                view, range, text, selected, window, view_cx,
            );
        });
    });
}

fn assert_virtual_document(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    expected_text: &str,
    expected_marked: std::ops::Range<usize>,
    expected_selected: std::ops::Range<usize>,
) -> Result<(), String> {
    let actual = virtual_snapshot(view, cx, expected_text.encode_utf16().count());
    let expected = (
        Some(expected_text.to_string()),
        Some(expected_marked),
        Some(expected_selected),
    );
    if actual == expected {
        return Ok(());
    }
    Err(format!(
        "virtual IME document was {actual:?}, expected {expected:?}"
    ))
}

fn virtual_snapshot(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    document_len: usize,
) -> (
    Option<String>,
    Option<std::ops::Range<usize>>,
    Option<std::ops::Range<usize>>,
) {
    cx.update(|window, app| {
        view.update(app, |view, view_cx| {
            let mut adjusted = None;
            let text = EntityInputHandler::text_for_range(
                view,
                0..document_len,
                &mut adjusted,
                window,
                view_cx,
            );
            let selected = EntityInputHandler::selected_text_range(view, false, window, view_cx)
                .map(|selection| selection.range);
            let marked = EntityInputHandler::marked_text_range(view, window, view_cx);
            (text, marked, selected)
        })
    })
}

fn assert_preedit_caret(view: &Entity<GpuiKucView>, cx: &TestAppContext) -> Result<(), String> {
    let preedit = cx.read(|app| {
        view.read(app)
            .frame
            .as_ref()
            .and_then(|frame| frame.composer.text_surface.preedit.clone())
    });
    let preedit = preedit.ok_or("KUC did not render the Japanese preedit")?;
    let first = preedit
        .rects
        .first()
        .ok_or("KUC preedit has no first glyph")?;
    let second = preedit
        .rects
        .get(1)
        .ok_or("KUC preedit has no second glyph")?;
    let first_right = first.x.saturating_add_unsigned(first.width);
    if preedit.caret.x == first_right && second.x == first_right {
        return Ok(());
    }
    Err(format!(
        "preedit caret was {:?}, first={first:?}, second={second:?}",
        preedit.caret
    ))
}

fn commit_marked_text(
    view: &Entity<GpuiKucView>,
    cx: &mut gpui::VisualTestContext,
    range: Option<std::ops::Range<usize>>,
    text: &str,
) {
    cx.update(|window, app| {
        view.update(app, |view, view_cx| {
            EntityInputHandler::replace_text_in_range(view, range, text, window, view_cx);
        });
    });
}

fn assert_committed_draft(
    view: &Entity<GpuiKucView>,
    cx: &TestAppContext,
    expected: &str,
) -> Result<(), String> {
    let draft = cx.read(|app| view.read(app).draft.clone());
    if draft == expected {
        return Ok(());
    }
    Err(format!(
        "marked text committed as {draft:?}, expected {expected:?}"
    ))
}

fn surface(draft: &str) -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.draft_mut().set_text(draft);
    ChatUiSurface::from_render_model(&session.render_model())
}
