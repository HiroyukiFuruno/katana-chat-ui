use super::super::{
    ChatUiRuntime, ChatUiRuntimeCompositionPhase, ChatUiRuntimeEffect, ChatUiRuntimeFrame,
    ChatUiRuntimeInput, ChatUiRuntimeKey,
};
use katana_ui_core::atom::{TextAreaAction, TextAreaSelection};
use katana_ui_core::text_surface::TextSurfaceAction;

use super::helpers::{
    VIEWPORT, busy_composer_surface, focus, focused_runtime, has_clipboard, has_draft,
    permission_surface, submit_count, surface,
};

#[test]
fn ime_composition_rasters_preedit_and_commits_draft() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("prefix");
    let mut runtime = focused_runtime(&surface)?;
    let before = runtime.render(&surface, VIEWPORT)?;
    runtime.dispatch(
        &before,
        ChatUiRuntimeInput::ImeComposition {
            phase: ChatUiRuntimeCompositionPhase::Start,
            preedit: "日本".to_string(),
            caret: "日".len(),
        },
    )?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let preedit = frame
        .composer
        .text_surface
        .preedit
        .as_ref()
        .ok_or_else(|| std::io::Error::other("missing IME preedit frame"))?;

    assert_eq!("日本", preedit.text);
    assert!(!preedit.rects.is_empty());
    let effects = runtime.dispatch(&frame, ChatUiRuntimeInput::ImeCommit("日本".to_string()))?;
    assert!(has_draft(&effects, "prefix日本"));
    Ok(())
}

#[test]
fn ime_cancel_removes_composition_without_empty_commit() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("prefix");
    let mut runtime = focused_runtime(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let _ = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::ImeComposition {
            phase: ChatUiRuntimeCompositionPhase::Update,
            preedit: "変換".to_string(),
            caret: "変".len(),
        },
    )?;
    let composing = runtime.render(&surface, VIEWPORT)?;
    assert!(composing.composer.text_surface.preedit.is_some());
    let effects = runtime.dispatch(&composing, ChatUiRuntimeInput::ImeCancel)?;
    let cancelled = runtime.render(&surface, VIEWPORT)?;

    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, ChatUiRuntimeEffect::Repaint))
    );
    assert!(cancelled.composer.text_surface.preedit.is_none());
    Ok(())
}

#[test]
fn permission_modal_blocks_keyboard_ime_clipboard_and_pointer_background_input()
-> Result<(), Box<dyn std::error::Error>> {
    let mut surface = surface("draft");
    let mut runtime = focused_runtime(&surface)?;
    start_preedit(&mut runtime, &surface)?;
    surface.vendor_bar.permission_request = permission_surface().vendor_bar.permission_request;
    let modal_frame = runtime.render(&surface, VIEWPORT)?;
    let bounds = modal_frame.composer.bounds;
    let selection = (
        modal_frame.composer.text_surface.selection_start,
        modal_frame.composer.text_surface.selection_end,
    );
    for input in permission_blocked_inputs(bounds) {
        assert_modal_input_is_ignored(&mut runtime, &modal_frame, input)?;
    }
    assert_pending_composer_unchanged(&mut runtime, &surface, selection)?;
    cancel_preedit_and_lose_focus(&mut runtime, &surface)?;
    Ok(())
}

fn start_preedit(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, VIEWPORT)?;
    let _ = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::ImeComposition {
            phase: ChatUiRuntimeCompositionPhase::Start,
            preedit: "変換".to_string(),
            caret: "変".len(),
        },
    )?;
    Ok(())
}

fn permission_blocked_inputs(
    bounds: katana_ui_core::render_model::UiRect,
) -> Vec<ChatUiRuntimeInput> {
    let point = (bounds.x as f32 + 8.0, bounds.y as f32 + 8.0);
    vec![
        ChatUiRuntimeInput::Text("typed".to_string()),
        ChatUiRuntimeInput::ClipboardPaste("pasted".to_string()),
        ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Backspace),
        ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Enter {
            shift: false,
            command: false,
        }),
        ChatUiRuntimeInput::ImeComposition {
            phase: ChatUiRuntimeCompositionPhase::Update,
            preedit: "replacement".to_string(),
            caret: 4,
        },
        ChatUiRuntimeInput::ImeCommit("committed".to_string()),
        ChatUiRuntimeInput::ImeCommitRange {
            start_byte: 0,
            end_byte: 0,
            text: "range".to_string(),
        },
        ChatUiRuntimeInput::ClipboardCut,
        ChatUiRuntimeInput::ClipboardCopy,
        ChatUiRuntimeInput::PointerDrag {
            x: point.0 + 80.0,
            y: point.1,
        },
        ChatUiRuntimeInput::PointerRelease,
    ]
}

fn assert_modal_input_is_ignored(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    input: ChatUiRuntimeInput,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(runtime.dispatch(frame, input)?.is_empty());
    Ok(())
}

fn assert_pending_composer_unchanged(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
    selection: (usize, usize),
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, VIEWPORT)?;
    assert_eq!(runtime.composer.state().text_area.value, "draft");
    assert_eq!(
        (
            frame.composer.text_surface.selection_start,
            frame.composer.text_surface.selection_end
        ),
        selection
    );
    assert_eq!(
        frame
            .composer
            .text_surface
            .preedit
            .as_ref()
            .map(|preedit| preedit.text.as_str()),
        Some("変換")
    );
    Ok(())
}

fn cancel_preedit_and_lose_focus(
    runtime: &mut ChatUiRuntime,
    surface: &crate::ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, VIEWPORT)?;
    let _ = runtime.dispatch(&frame, ChatUiRuntimeInput::ImeCancel)?;
    let frame = runtime.render(surface, VIEWPORT)?;
    assert!(frame.composer.text_surface.preedit.is_none());
    let _ = runtime.dispatch(&frame, ChatUiRuntimeInput::FocusLost)?;
    assert!(!runtime.composer.state().text_area.focused);
    Ok(())
}

#[test]
fn ime_commit_range_replaces_the_host_selected_byte_range() -> Result<(), Box<dyn std::error::Error>>
{
    let surface = surface("hello");
    let mut runtime = focused_runtime(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let effects = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::ImeCommitRange {
            start_byte: 1,
            end_byte: 4,
            text: "i".to_string(),
        },
    )?;

    assert!(has_draft(&effects, "hio"));
    Ok(())
}

#[test]
fn enter_inserts_newline_and_command_enter_submits_once() -> Result<(), Box<dyn std::error::Error>>
{
    let surface = surface("hello");
    let mut runtime = focused_runtime(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let newline = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Enter {
            shift: false,
            command: false,
        }),
    )?;

    assert!(has_draft(&newline, "hello\n"));
    assert_eq!(0, submit_count(&newline));
    let submit = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Enter {
            shift: false,
            command: true,
        }),
    )?;
    assert_eq!(1, submit_count(&submit));
    Ok(())
}

#[test]
fn command_enter_does_not_submit_while_assistant_streams() -> Result<(), Box<dyn std::error::Error>>
{
    let surface = busy_composer_surface()?;
    let mut runtime = focused_runtime(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    for shift in [false, true] {
        let effects = runtime.dispatch(
            &frame,
            ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Enter {
                shift,
                command: true,
            }),
        )?;
        assert_eq!(0, submit_count(&effects));
        assert!(effects.is_empty());
        assert_eq!(runtime.composer.state().text_area.value, "keep this draft");
    }
    Ok(())
}

#[test]
fn forward_delete_at_end_and_empty_is_a_draft_noop() -> Result<(), Box<dyn std::error::Error>> {
    for value in ["", "hello", "末尾"] {
        let surface = surface(value);
        let mut runtime = focused_runtime(&surface)?;
        let frame = runtime.render(&surface, VIEWPORT)?;
        let effects =
            runtime.dispatch(&frame, ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Delete))?;
        assert!(
            effects
                .iter()
                .all(|effect| !matches!(effect, ChatUiRuntimeEffect::DraftChanged(_)))
        );
        assert_eq!(runtime.composer.state().text_area.value, value);
        assert_eq!(runtime.composer.state().text_area.caret, value.len());
    }
    Ok(())
}

#[test]
fn forward_delete_removes_one_grapheme_and_keeps_caret() -> Result<(), Box<dyn std::error::Error>> {
    for (value, caret, expected) in [
        ("a👩‍💻b", 1, "ab"),
        ("ae\u{301}b", 1, "ab"),
        ("a\nb", 1, "ab"),
    ] {
        let surface = surface(value);
        let mut runtime = focused_runtime(&surface)?;
        place_caret(&mut runtime, caret);
        let frame = runtime.render(&surface, VIEWPORT)?;
        let effects =
            runtime.dispatch(&frame, ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Delete))?;
        assert!(has_draft(&effects, expected));
        assert_eq!(runtime.composer.state().text_area.value, expected);
        assert_eq!(runtime.composer.state().text_area.caret, caret);
    }
    Ok(())
}

#[test]
fn forward_delete_removes_forward_and_reverse_selections_only()
-> Result<(), Box<dyn std::error::Error>> {
    let value = "a👩‍💻b\nc";
    let end = 1 + "👩‍💻".len();
    for (start, finish) in [(1, end), (end, 1)] {
        let surface = surface(value);
        let mut runtime = focused_runtime(&surface)?;
        select_range(&mut runtime, start, finish);
        let frame = runtime.render(&surface, VIEWPORT)?;
        let effects =
            runtime.dispatch(&frame, ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Delete))?;
        assert!(has_draft(&effects, "ab\nc"));
        assert_eq!(runtime.composer.state().text_area.value, "ab\nc");
        assert_eq!(runtime.composer.state().text_area.caret, 1);
        assert_eq!(runtime.composer.state().text_area.selection.start, 1);
        assert_eq!(runtime.composer.state().text_area.selection.end, 1);
    }
    Ok(())
}

#[test]
fn backspace_removes_forward_reverse_and_whole_selections() -> Result<(), Box<dyn std::error::Error>>
{
    let value = "a👩‍💻b\nc";
    let end = 1 + "👩‍💻b".len();
    for (start, finish, expected) in [
        (1, end, "a\nc"),
        (end, 1, "a\nc"),
        (0, value.len(), ""),
        (value.len(), 0, ""),
    ] {
        let surface = surface(value);
        let mut runtime = focused_runtime(&surface)?;
        select_range(&mut runtime, start, finish);
        let frame = runtime.render(&surface, VIEWPORT)?;
        let effects =
            runtime.dispatch(&frame, ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Backspace))?;
        assert!(has_draft(&effects, expected));
        let state = &runtime.composer.state().text_area;
        assert_eq!(state.value, expected);
        assert_eq!(state.caret, start.min(finish));
        assert_eq!(state.selection.start, state.caret);
        assert_eq!(state.selection.end, state.caret);
    }
    Ok(())
}

#[test]
fn backspace_without_selection_keeps_previous_grapheme_behavior()
-> Result<(), Box<dyn std::error::Error>> {
    for (value, expected) in [("", ""), ("a👩‍💻", "a"), ("ae\u{301}", "a"), ("a\n", "a")] {
        let surface = surface(value);
        let mut runtime = focused_runtime(&surface)?;
        let frame = runtime.render(&surface, VIEWPORT)?;
        let effects =
            runtime.dispatch(&frame, ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Backspace))?;
        let state = &runtime.composer.state().text_area;
        assert_eq!(state.value, expected);
        assert_eq!(state.caret, expected.len());
        if value != expected {
            assert!(has_draft(&effects, expected));
        } else {
            assert!(
                effects
                    .iter()
                    .all(|effect| !matches!(effect, ChatUiRuntimeEffect::DraftChanged(_)))
            );
        }
    }
    Ok(())
}

fn place_caret(runtime: &mut ChatUiRuntime, caret: usize) {
    let _ = runtime
        .composer
        .apply_action(TextSurfaceAction::TextArea(TextAreaAction::Select(
            TextAreaSelection::collapsed(caret),
        )));
}

fn select_range(runtime: &mut ChatUiRuntime, start: usize, end: usize) {
    let _ = runtime
        .composer
        .apply_action(TextSurfaceAction::TextArea(TextAreaAction::Select(
            TextAreaSelection { start, end },
        )));
}

#[test]
fn composer_scroll_crops_rasterized_multiline_draft() -> Result<(), Box<dyn std::error::Error>> {
    let draft = (0..32)
        .map(|line| format!("line {line:02}: visible text"))
        .collect::<Vec<_>>()
        .join("\n");
    let surface = surface(&draft);
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let before = runtime.render(&surface, VIEWPORT)?;
    let bounds = before.composer.bounds;
    let _ = runtime.dispatch(
        &before,
        ChatUiRuntimeInput::Scroll {
            x: bounds.x as f32 + 8.0,
            y: bounds.y as f32 + 8.0,
            delta_x: 0,
            delta_y: 40,
        },
    )?;
    let after = runtime.render(&surface, VIEWPORT)?;

    assert_eq!(40, after.composer.text_surface.viewport.scroll_y);
    assert!(composer_pixel_differences(&before, &after)? > 128);
    Ok(())
}

#[test]
fn unfocused_and_focus_lost_text_input_is_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;

    assert!(
        runtime
            .dispatch(&frame, ChatUiRuntimeInput::Text("!".to_string()))?
            .is_empty()
    );
    let _ = focus(&mut runtime, &surface)?;
    let focused_frame = runtime.render(&surface, VIEWPORT)?;
    let _ = runtime.dispatch(&focused_frame, ChatUiRuntimeInput::FocusLost)?;
    assert!(
        runtime
            .dispatch(
                &focused_frame,
                ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Backspace)
            )?
            .is_empty()
    );
    Ok(())
}

fn composer_pixel_differences(
    before: &super::super::ChatUiRuntimeFrame,
    after: &super::super::ChatUiRuntimeFrame,
) -> Result<usize, Box<dyn std::error::Error>> {
    let bounds = before.composer.bounds;
    let left = usize::try_from(bounds.x.max(0))?;
    let top = usize::try_from(bounds.y.max(0))?;
    let width = bounds.width as usize;
    let height = bounds.height as usize;
    Ok((top..top.saturating_add(height))
        .flat_map(|y| (left..left.saturating_add(width)).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let index = y
                .saturating_mul(before.rgba.width as usize)
                .saturating_add(*x);
            before.rgba.pixels.get(index) != after.rgba.pixels.get(index)
        })
        .count())
}

#[test]
fn selection_copy_and_cut_require_focus_and_selection() -> Result<(), Box<dyn std::error::Error>> {
    let surface = surface("hello");
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;

    assert!(
        runtime
            .dispatch(&frame, ChatUiRuntimeInput::ClipboardCut)?
            .is_empty()
    );
    let _ = focus(&mut runtime, &surface)?;
    let focused_frame = runtime.render(&surface, VIEWPORT)?;
    assert!(
        runtime
            .dispatch(&focused_frame, ChatUiRuntimeInput::ClipboardCut)?
            .is_empty()
    );
    let _ = runtime.dispatch(
        &focused_frame,
        ChatUiRuntimeInput::Key(ChatUiRuntimeKey::CommandA),
    )?;
    let copy = runtime.dispatch(&focused_frame, ChatUiRuntimeInput::ClipboardCopy)?;

    assert!(has_clipboard(&copy, "hello"));
    let cut = runtime.dispatch(&focused_frame, ChatUiRuntimeInput::ClipboardCut)?;
    assert!(has_clipboard(&cut, "hello"));
    assert!(has_draft(&cut, ""));
    Ok(())
}
