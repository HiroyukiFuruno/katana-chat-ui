use super::super::super::{
    ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeFrame, ChatUiRuntimeInput,
    ChatUiRuntimeViewport,
};
use super::super::helpers::{find_node, hit};
use crate::{ChatSession, ChatUiAction, ChatUiHistorySessionSurface, ChatUiSurface, ThinkingLog};
use katana_ui_core::raster_host::{Canvas, UiTreeHitRect, UiTreeRenderArea};
use katana_ui_core::render_model::UiDimension;
use katana_ui_core::theme::ThemeSnapshot;
use std::collections::HashMap;

const STORYBOOK_VIEWPORT: ChatUiRuntimeViewport = ChatUiRuntimeViewport::new(1280, 900, 1.0);

#[test]
fn one_line_message_keeps_its_existing_row_height() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Storybook fixture");
    session.draft_mut().set_text("hello");
    session.submit_draft()?;
    session.start_assistant_stream("world")?;
    session.finish_assistant_message()?;
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let mut runtime = ChatUiRuntime::light(&surface)?;

    let frame = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    for message_id in [1, 2] {
        let row = find_node(frame.tree.root(), &format!("chat.message.{message_id}.row"))
            .ok_or("missing one-line message row")?;
        assert_eq!(
            44,
            px(&row.props().common.height)?,
            "one-line message rows must not grow from counting card padding twice"
        );
    }
    Ok(())
}

#[test]
fn completed_empty_assistant_messages_keep_their_minimum_row_height()
-> Result<(), Box<dyn std::error::Error>> {
    for content in ["", " \n "] {
        let mut session = ChatSession::new();
        session.set_provider_configured("Storybook fixture");
        session.start_assistant_stream(content)?;
        session.finish_assistant_message()?;
        let surface = ChatUiSurface::from_render_model(&session.render_model());
        assert!(surface.message_list.messages[0].body.trim().is_empty());
        assert!(
            surface.message_list.messages[0]
                .status_label
                .trim()
                .is_empty()
        );
        assert!(surface.message_list.messages[0].activity.is_none());
        assert!(surface.message_list.messages[0].thinking.is_none());
        assert!(surface.message_list.messages[0].attachments.is_empty());
        assert!(surface.message_list.messages[0].outputs.is_empty());

        let mut runtime = ChatUiRuntime::light(&surface)?;
        let frame = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
        let row = find_node(frame.tree.root(), "chat.message.1.row")
            .ok_or("missing empty assistant message row")?;
        assert_eq!(44, px(&row.props().common.height)?);
    }
    Ok(())
}

#[test]
fn storybook_japanese_prompt_fits_its_measured_message_row()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = storybook_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, STORYBOOK_VIEWPORT)?;

    assert_message_content_fits_row(&runtime, &frame, 1)?;
    assert_thread_content_covers_rows(&frame, &[1, 2])?;
    assert_second_line_is_rendered(&runtime, &frame, "chat.message.1.body.0")?;
    Ok(())
}

#[test]
fn narrow_japanese_and_multiple_markdown_headings_fit_their_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = narrow_japanese_markdown_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let viewport = ChatUiRuntimeViewport::new(240, 420, 1.0);
    let frame = runtime.render(&surface, viewport)?;

    assert_message_content_fits_row(&runtime, &frame, 1)?;
    assert_message_content_fits_row(&runtime, &frame, 2)?;
    assert_thread_content_covers_rows(&frame, &[1, 2])?;
    Ok(())
}

#[test]
fn measured_multiline_content_remains_reachable_at_thread_scroll_max()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = narrow_japanese_markdown_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let viewport = ChatUiRuntimeViewport::new(240, 320, 1.0);
    let before = runtime.render(&surface, viewport)?;
    let thread = find_node(before.tree.root(), "chat.thread").ok_or("missing thread")?;
    assert!(thread.props().scroll_area.content_height > thread.props().scroll_area.viewport_height);

    dispatch_scroll_to_end(&mut runtime, &before, 8.0, 40.0)?;
    let after = runtime.render(&surface, viewport)?;
    let thread = find_node(after.tree.root(), "chat.thread").ok_or("missing scrolled thread")?;
    let maximum = thread
        .props()
        .scroll_area
        .content_height
        .saturating_sub(thread.props().scroll_area.viewport_height);

    assert_eq!(
        maximum,
        thread.props().scroll_area.offset_y,
        "runtime tree offset is {} and composer begins at {}",
        runtime.tree_scroll_y,
        after.composer.bounds.y
    );
    assert_last_message_text_is_visible(&runtime, &after, 2)?;
    Ok(())
}

#[test]
fn expanded_thinking_content_fits_its_measured_row() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Storybook fixture");
    session.start_assistant_stream_with_thinking(
        "",
        ThinkingLog::running(
            "長い推論の状態を確認します",
            vec![
                "最初の項目は画面幅に合わせて複数行へ折り返されます。".into(),
                "最後の項目もスクロール末尾まで表示されます。".into(),
            ],
        ),
    )?;
    let surface = ChatUiSurface::from_render_model(&session.render_model());
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, STORYBOOK_VIEWPORT)?;

    assert_message_content_fits_row(&runtime, &frame, 1)?;
    assert_thread_content_covers_rows(&frame, &[1])?;
    assert_node_ink_is_visible(&runtime, &frame, "chat.message.1.thinking.entries")?;
    Ok(())
}

#[test]
fn storybook_history_is_immediately_visible_below_the_header()
-> Result<(), Box<dyn std::error::Error>> {
    let mut surface = long_storybook_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let before = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    let thread_offset = scroll_thread_to_end(&mut runtime, &surface, &before)?;
    surface = open_storybook_history(&mut runtime, &surface, 1)?;
    let opened = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    assert_history_row_visible(&runtime, &opened, 0)?;
    assert_history_viewport_is_bounded(&opened)?;
    assert_eq!(thread_offset, thread_scroll_offset(&runtime, &opened)?);
    assert!(
        opened
            .action_hits
            .iter()
            .any(|hit| hit.action.payload.contains("ToggleHistory"))
    );
    assert_eq!(before.composer.bounds, opened.composer.bounds);
    Ok(())
}

#[test]
fn visible_history_hides_thread_pixels_and_background_actions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut first_surface = long_storybook_surface()?;
    let mut first_runtime = ChatUiRuntime::light(&first_surface)?;
    first_surface = open_storybook_history(&mut first_runtime, &first_surface, 1)?;
    let first = first_runtime.render(&first_surface, STORYBOOK_VIEWPORT)?;

    let mut second_surface = long_storybook_surface()?;
    second_surface.message_list.messages[0].body =
        "# A different conversation\n\nThis body must never appear through the open history overlay."
            .to_string();
    second_surface = surface_with_history(&second_surface, 1);
    let mut second_runtime = ChatUiRuntime::light(&second_surface)?;
    let second = second_runtime.render(&second_surface, STORYBOOK_VIEWPORT)?;

    let viewport = history_scroll_rect(&first)?;
    assert_eq!(
        viewport,
        history_scroll_rect(&second)?,
        "history viewport geometry must not depend on conversation content"
    );
    assert_history_viewport_pixels_match(&first, &second, viewport);
    assert_history_hides_thread(&first);

    let mut dark_first_runtime = ChatUiRuntime::new(ThemeSnapshot::dark(), &first_surface)?;
    let mut dark_second_runtime = ChatUiRuntime::new(ThemeSnapshot::dark(), &second_surface)?;
    let dark_first = dark_first_runtime.render(&first_surface, STORYBOOK_VIEWPORT)?;
    let dark_second = dark_second_runtime.render(&second_surface, STORYBOOK_VIEWPORT)?;
    assert_history_viewport_pixels_match(
        &dark_first,
        &dark_second,
        history_scroll_rect(&dark_first)?,
    );
    assert_history_hides_thread(&dark_first);
    Ok(())
}

fn assert_history_hides_thread(frame: &ChatUiRuntimeFrame) {
    assert!(
        frame
            .action_hits
            .iter()
            .all(|hit| { !hit.action.target.as_str().starts_with("chat.message.") })
    );
    assert!(find_node(frame.tree.root(), "chat.frame.thread").is_none());
    assert!(find_node(frame.tree.root(), "chat.history.empty-label").is_none());
}

#[test]
fn empty_history_label_is_only_rendered_without_sessions() -> Result<(), Box<dyn std::error::Error>>
{
    let mut empty_surface = long_storybook_surface()?;
    empty_surface.history_panel.visible = true;
    let mut empty_runtime = ChatUiRuntime::light(&empty_surface)?;
    let empty = empty_runtime.render(&empty_surface, STORYBOOK_VIEWPORT)?;
    assert!(find_node(empty.tree.root(), "chat.history.empty-label").is_some());

    let session_surface = surface_with_history(&empty_surface, 1);
    let mut session_runtime = ChatUiRuntime::light(&session_surface)?;
    let session = session_runtime.render(&session_surface, STORYBOOK_VIEWPORT)?;
    assert!(find_node(session.tree.root(), "chat.history.empty-label").is_none());
    Ok(())
}

#[test]
fn narrow_history_rows_wrap_all_visible_fields_without_clipping()
-> Result<(), Box<dyn std::error::Error>> {
    let mut surface = long_storybook_surface()?;
    surface.history_panel.visible = true;
    surface.history_panel.sessions = vec![ChatUiHistorySessionSurface {
        session_id: "long-history".to_string(),
        title: "A deliberately long history title that must wrap inside the narrow selection row"
            .to_string(),
        provider_id: "fixture-provider".to_string(),
        provider_label: "A deliberately long provider label".to_string(),
        updated_at_label: "updated just now in the narrow fixture viewport".to_string(),
        preview: "A deliberately long preview that must remain visible below the wrapped title and metadata"
            .to_string(),
    }];
    let viewport = ChatUiRuntimeViewport::new(320, 500, 1.0);
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, viewport)?;
    let row =
        find_node(frame.tree.root(), "chat.history.session.0").ok_or("missing history row")?;
    assert!(
        row.props().label.is_empty(),
        "row label must not duplicate the wrapped title"
    );
    for suffix in ["title", "meta", "preview"] {
        assert_wrapped_history_field(&runtime, &frame, suffix)?;
    }
    assert_history_selection_covers_wrapped_row(&mut runtime, &frame, "long-history")?;
    Ok(())
}

fn assert_history_selection_covers_wrapped_row(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let action = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(session_id))
        .ok_or("missing long history selection action")?;
    for suffix in ["title", "meta", "preview"] {
        assert_history_child_inside_action(frame, action, 0, suffix)?;
    }
    dispatch_history_preview(runtime, frame, session_id)
}

fn assert_history_child_inside_action(
    frame: &ChatUiRuntimeFrame,
    action: &katana_ui_core::raster_host::UiTreeHostActionHit,
    index: usize,
    suffix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let node_id = format!("chat.history.session.{index}.{suffix}");
    let child = hit(&frame.node_hits, &node_id).ok_or("missing history child hit")?;
    assert!(
        child.rect.x >= action.rect.x
            && child.rect.y >= action.rect.y
            && child.rect.x + child.rect.width <= action.rect.x + action.rect.width
            && child.rect.y + child.rect.height <= action.rect.y + action.rect.height,
        "{node_id} exceeds action: child={:?} action={:?}",
        child.rect,
        action.rect
    );
    Ok(())
}

#[test]
fn wrapped_history_rows_keep_adjacent_bounds_and_last_preview_selectable()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = long_wrapped_history_surface()?;
    let viewport = ChatUiRuntimeViewport::new(320, 500, 1.0);
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let full_content_height = full_history_content_height(&mut runtime, &surface, viewport.width)?;
    let opened = runtime.render(&surface, viewport)?;
    let action0 = history_action(&opened, 0)?;
    let action1 = history_action(&opened, 1)?;
    assert!(action0.rect.y + action0.rect.height <= action1.rect.y);
    for index in [0, 1] {
        let action = history_action(&opened, index)?;
        for suffix in ["title", "meta", "preview"] {
            assert_history_child_inside_action(&opened, action, index, suffix)?;
        }
    }
    scroll_history_to_end(&mut runtime, &opened)?;
    let scrolled = runtime.render(&surface, viewport)?;
    let history_scroll = find_node(scrolled.tree.root(), "chat.history.scroll")
        .ok_or("missing history scroll node")?;
    assert_eq!(
        full_content_height as u32,
        history_scroll.props().scroll_area.content_height,
        "history content height must retain full unbounded KUC hit extent"
    );
    assert_last_history_preview_selectable(&mut runtime, &scrolled)?;
    Ok(())
}

fn full_history_content_height(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    width: u32,
) -> Result<usize, Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, ChatUiRuntimeViewport::new(width, 2000, 1.0))?;
    let scroll = find_node(frame.tree.root(), "chat.history.scroll")
        .ok_or("missing full history scroll node")?;
    assert_eq!(
        scroll.props().scroll_area.content_height,
        scroll.props().scroll_area.viewport_height,
        "the reference viewport must contain the entire history fixture"
    );
    Ok(history_content_bottom(&frame)?.saturating_sub(history_scroll_rect(&frame)?.y as usize))
}

fn history_content_bottom(frame: &ChatUiRuntimeFrame) -> Result<usize, Box<dyn std::error::Error>> {
    frame
        .node_hits
        .iter()
        .filter(|hit| hit.node_id.as_str().starts_with("chat.history.session."))
        .map(|hit| hit.rect.y + hit.rect.height)
        .max()
        .ok_or_else(|| "missing history child hit extent".into())
}

fn long_wrapped_history_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut surface = surface_with_history(&long_storybook_surface()?, 20);
    for (index, session) in surface.history_panel.sessions.iter_mut().enumerate() {
        session.title = format!("Short history {index}");
        session.provider_label = "fixture".to_string();
        session.updated_at_label = "now".to_string();
        session.preview = format!("Preview {index}");
    }
    for index in [0, 19] {
        let session = &mut surface.history_panel.sessions[index];
        session.title = format!(
            "Long wrapped history title {index} that takes multiple lines in a narrow viewport"
        );
        session.provider_label = "A long provider label for wrapped metadata".to_string();
        session.updated_at_label = "updated in the long fixture timestamp".to_string();
        session.preview =
            "A long preview that must remain clickable at the bottom of the row".to_string();
    }
    Ok(surface)
}

fn assert_last_history_preview_selectable(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let last_action = history_action(frame, 19)?;
    for suffix in ["title", "meta", "preview"] {
        assert_history_child_inside_action(frame, last_action, 19, suffix)?;
    }
    let preview = hit(&frame.node_hits, "chat.history.session.19.preview")
        .ok_or("missing last preview hit")?;
    let history_viewport = history_scroll_rect(frame)?;
    assert!(
        preview.rect.y + preview.rect.height
            <= history_viewport.y as usize + history_viewport.height as usize
    );
    let (x, y) = preview.rect.center_point();
    let effects = runtime.dispatch(
        frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::SelectHistory { session_id })
            if session_id == "storybook-history-19"
    )));
    Ok(())
}

fn history_action(
    frame: &ChatUiRuntimeFrame,
    index: usize,
) -> Result<&katana_ui_core::raster_host::UiTreeHostActionHit, Box<dyn std::error::Error>> {
    let target = format!("chat.history.session.{index}");
    frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == target)
        .ok_or_else(|| format!("missing history action {target}").into())
}

fn dispatch_history_preview(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let preview =
        hit(&frame.node_hits, "chat.history.session.0.preview").ok_or("missing preview hit")?;
    let (x, y) = preview.rect.center_point();
    let effects = runtime.dispatch(
        frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::SelectHistory { session_id: selected })
            if selected == session_id
    )));
    Ok(())
}

fn assert_wrapped_history_field(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    suffix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let node_id = format!("chat.history.session.0.{suffix}");
    let node = find_node(frame.tree.root(), &node_id).ok_or("missing wrapped history field")?;
    assert_eq!(
        katana_ui_core::render_model::UiTextWrapMode::Wrap,
        node.props().text.wrap,
        "{node_id} must use public KUC wrapping"
    );
    let hit = hit(&frame.node_hits, &node_id).ok_or("missing wrapped history field bounds")?;
    assert!(
        hit.rect.height >= 40,
        "{node_id} did not grow for its long value"
    );
    assert!(has_non_background_ink(runtime, frame, hit.rect));
    Ok(())
}

#[test]
fn long_storybook_history_scrolls_independently_and_selects_last_row()
-> Result<(), Box<dyn std::error::Error>> {
    let mut surface = long_storybook_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let before = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    let thread_offset = scroll_thread_to_end(&mut runtime, &surface, &before)?;
    surface = open_storybook_history(&mut runtime, &surface, 40)?;
    let opened = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    assert_history_row_visible(&runtime, &opened, 0)?;
    let opened_thread_offset = thread_scroll_offset(&runtime, &opened)?;
    scroll_history_to_end(&mut runtime, &opened)?;
    let scrolled = runtime.render(&surface, STORYBOOK_VIEWPORT)?;

    assert_eq!(thread_offset, opened_thread_offset);
    assert_eq!(thread_offset, thread_scroll_offset(&runtime, &scrolled)?);
    let history_scroll = find_node(scrolled.tree.root(), "chat.history.scroll")
        .ok_or("missing history scroll node")?;
    assert!(
        history_scroll.props().scroll_area.offset_y > 0,
        "history offset={} content={} viewport={} bounds={:?}",
        history_scroll.props().scroll_area.offset_y,
        history_scroll.props().scroll_area.content_height,
        history_scroll.props().scroll_area.viewport_height,
        history_scroll_rect(&scrolled)?
    );
    assert_history_row_visible(&runtime, &scrolled, 39)?;
    assert_history_viewport_is_bounded(&scrolled)?;
    assert_selection_action(&mut runtime, &scrolled, "storybook-history-39")?;
    Ok(())
}

#[test]
fn closing_or_replacing_history_resets_its_scroll_offset() -> Result<(), Box<dyn std::error::Error>>
{
    let mut surface = long_storybook_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    surface = open_storybook_history(&mut runtime, &surface, 40)?;
    let opened = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    scroll_history_to_end(&mut runtime, &opened)?;
    let scrolled = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    assert!(history_scroll_offset(&scrolled)? > 0);

    let mut changed = surface_with_history(&surface, 40);
    changed.history_panel.sessions[0].session_id = "replacement-session".to_string();
    let replaced = runtime.render(&changed, STORYBOOK_VIEWPORT)?;
    assert_eq!(0, history_scroll_offset(&replaced)?);

    scroll_history_to_end(&mut runtime, &replaced)?;
    let scrolled = runtime.render(&changed, STORYBOOK_VIEWPORT)?;
    assert!(history_scroll_offset(&scrolled)? > 0);
    let mut closed = surface.clone();
    closed.history_panel.visible = false;
    runtime.render(&closed, STORYBOOK_VIEWPORT)?;
    let reopened = surface_with_history(&closed, 40);
    let reset = runtime.render(&reopened, STORYBOOK_VIEWPORT)?;
    assert_eq!(0, history_scroll_offset(&reset)?);
    Ok(())
}

#[test]
fn resizing_history_viewport_clamps_its_scroll_offset() -> Result<(), Box<dyn std::error::Error>> {
    let mut surface = long_storybook_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    surface = open_storybook_history(&mut runtime, &surface, 40)?;
    let opened = runtime.render(&surface, STORYBOOK_VIEWPORT)?;
    scroll_history_to_end(&mut runtime, &opened)?;
    let scrolled = runtime.render(&surface, STORYBOOK_VIEWPORT)?;

    let resized_viewport = ChatUiRuntimeViewport::new(1280, 1000, 1.0);
    let resized = runtime.render(&surface, resized_viewport)?;
    let scroll = find_node(resized.tree.root(), "chat.history.scroll")
        .ok_or("missing resized history scroll node")?;
    let maximum = scroll
        .props()
        .scroll_area
        .content_height
        .saturating_sub(scroll.props().scroll_area.viewport_height);
    assert_eq!(maximum, scroll.props().scroll_area.offset_y);
    assert!(history_scroll_offset(&scrolled)? >= maximum);
    assert_history_row_visible(&runtime, &resized, 39)?;
    Ok(())
}

fn open_storybook_history(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    session_count: usize,
) -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let frame = runtime.render(surface, STORYBOOK_VIEWPORT)?;
    let history_action = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains("ToggleHistory"))
        .ok_or("missing Storybook history action")?;
    let (x, y) = history_action.center_point();
    let effects = runtime.dispatch(
        &frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::ToggleHistory)
    )));
    Ok(surface_with_history(surface, session_count))
}

fn surface_with_history(surface: &ChatUiSurface, session_count: usize) -> ChatUiSurface {
    let mut surface = surface.clone();
    surface.history_panel.visible = true;
    surface.history_panel.sessions = (0..session_count)
        .map(|index| ChatUiHistorySessionSurface {
            session_id: format!("storybook-history-{index}"),
            title: format!("KCU Storybook reference {index}"),
            provider_id: surface.vendor_bar.active_vendor_id.clone(),
            provider_label: surface.vendor_bar.active_vendor_label.clone(),
            updated_at_label: "fixture".to_string(),
            preview: "KUC 共通 runtime の基準会話".to_string(),
        })
        .collect();
    surface
}

fn scroll_thread_to_end(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
    frame: &ChatUiRuntimeFrame,
) -> Result<u32, Box<dyn std::error::Error>> {
    dispatch_scroll_to_end(runtime, frame, 640.0, 500.0)?;
    let rendered = runtime.render(surface, STORYBOOK_VIEWPORT)?;
    thread_scroll_offset(runtime, &rendered)
}

fn dispatch_scroll_to_end(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    x: f32,
    y: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    runtime.dispatch(
        frame,
        ChatUiRuntimeInput::Scroll {
            x,
            y,
            delta_x: 0,
            delta_y: i32::MAX,
        },
    )?;
    Ok(())
}

fn thread_scroll_offset(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(find_node(frame.tree.root(), "chat.thread")
        .map(|node| node.props().scroll_area.offset_y)
        .unwrap_or(runtime.tree_scroll_y))
}

fn history_scroll_offset(frame: &ChatUiRuntimeFrame) -> Result<u32, Box<dyn std::error::Error>> {
    find_node(frame.tree.root(), "chat.history.scroll")
        .map(|node| node.props().scroll_area.offset_y)
        .ok_or_else(|| "missing KUC history scroll area".into())
}

fn scroll_history_to_end(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let viewport = history_scroll_rect(frame)?;
    let (x, y) = rect_center(viewport);
    runtime.dispatch(
        frame,
        ChatUiRuntimeInput::Scroll {
            x: x as f32,
            y: y as f32,
            delta_x: 0,
            delta_y: i32::MAX,
        },
    )?;
    Ok(())
}

fn assert_history_viewport_is_bounded(
    frame: &ChatUiRuntimeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    let viewport = history_scroll_rect(frame)?;
    assert_eq!(40, viewport.y);
    assert!(viewport.y.saturating_add_unsigned(viewport.height) <= frame.composer.bounds.y);
    let history_toggle = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains("ToggleHistory"))
        .ok_or("history toggle action is not reachable")?;
    assert!(history_toggle.rect.y + history_toggle.rect.height <= viewport.y as usize);
    Ok(())
}

fn assert_selection_action(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let action = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.payload.contains(session_id))
        .ok_or("visible Storybook history row has no selection action")?;
    let (x, y) = action.center_point();
    let effects = runtime.dispatch(
        frame,
        ChatUiRuntimeInput::PointerPress {
            x,
            y,
            extend_selection: false,
        },
    )?;
    assert!(effects.iter().any(|effect| matches!(
        effect,
        ChatUiRuntimeEffect::Action(ChatUiAction::SelectHistory { session_id: selected })
            if selected == session_id
    )));
    Ok(())
}

fn storybook_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_title("KCU Storybook");
    session.set_provider_configured("Storybook fixture");
    session
        .draft_mut()
        .set_text("KCU Storybook では何を確認できますか？");
    session.submit_draft()?;
    session.start_assistant_stream("# KCU Storybook\n\n共通 runtime を通る対話確認面です。")?;
    session.finish_assistant_message()?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn long_storybook_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let paragraphs = (0..72)
        .map(|index| {
            format!(
                "## Storybook section {index}\n\nThis long deterministic Storybook fixture keeps history below the visible thread until the user scrolls."
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut session = ChatSession::new();
    session.set_title("KCU Storybook");
    session.set_provider_configured("Storybook fixture");
    session.start_assistant_stream(format!("# Long Storybook conversation\n\n{paragraphs}"))?;
    session.finish_assistant_message()?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn narrow_japanese_markdown_surface() -> Result<ChatUiSurface, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Storybook fixture");
    session
        .draft_mut()
        .set_text("日本語の長い依頼です。表示幅が狭い場合にも本文のすべての行を確認できますか？");
    session.submit_draft()?;
    session.start_assistant_stream(
        "# 日本語の回答\n\n本文を複数行で表示します。\n\n## 次の見出し\n\nこちらも折り返される長い説明です。",
    )?;
    session.finish_assistant_message()?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn assert_last_message_text_is_visible(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    message_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefix = format!("chat.message.{message_id}.body.");
    let document_last = document_hits(runtime, frame)
        .into_iter()
        .filter(|hit| hit.node_id.as_str().starts_with(&prefix))
        .max_by_key(|hit| hit.rect.y.saturating_add(hit.rect.height))
        .ok_or("no document body text at thread scroll max")?;
    let last = frame
        .node_hits
        .iter()
        .find(|hit| hit.node_id == document_last.node_id)
        .ok_or("no rendered body text at thread scroll max")?;
    let bottom = last.rect.y.saturating_add(last.rect.height);
    assert!(bottom <= frame.composer.bounds.y as usize);
    assert_eq!(document_last.rect.height, last.rect.height);
    let background = pixel(frame, 0, 0);
    let has_ink = (last.rect.y..bottom).any(|y| {
        (last.rect.x..last.rect.x.saturating_add(last.rect.width)).any(|x| {
            x < frame.rgba.width as usize
                && y < frame.rgba.height as usize
                && pixel(frame, x, y) != background
        })
    });
    assert!(has_ink, "last visible Markdown block has no rendered text");
    Ok(())
}

fn assert_message_content_fits_row(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    message_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let row_id = format!("chat.message.{message_id}.row");
    let row = find_node(frame.tree.root(), &row_id).ok_or("missing message row")?;
    let row_height = px(&row.props().common.height)?;
    let (top_padding, bottom_padding) = message_card_padding(frame, message_id)?;
    let row_top = row_top_for(frame, message_id)?;
    let (first_y, content_bottom) = message_content_extent(runtime, frame, message_id)?;
    assert_message_extent_in_row(
        &row_id,
        row_top,
        row_height,
        top_padding,
        bottom_padding,
        first_y,
        content_bottom,
    );
    Ok(())
}

fn message_card_padding(
    frame: &ChatUiRuntimeFrame,
    message_id: u64,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let card = find_node(frame.tree.root(), &format!("chat.message.{message_id}"))
        .ok_or("missing message card")?;
    Ok((
        px(&card.props().common.padding.top)?,
        px(&card.props().common.padding.bottom)?,
    ))
}

fn message_content_extent(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    message_id: u64,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let content_prefix = format!("chat.message.{message_id}.");
    let measured_hits = document_hits(runtime, frame);
    let content_hits = measured_hits
        .iter()
        .filter(|hit| hit.node_id.as_str().starts_with(&content_prefix))
        .collect::<Vec<_>>();
    let first_y = content_hits
        .iter()
        .map(|hit| hit.rect.y)
        .min()
        .ok_or("message must have a rendered descendant")?;
    let content_bottom = content_hits
        .iter()
        .map(|hit| hit.rect.y.saturating_add(hit.rect.height))
        .max()
        .ok_or("message must have a rendered descendant")?;
    Ok((first_y, content_bottom))
}

fn assert_message_extent_in_row(
    row_id: &str,
    row_top: usize,
    row_height: usize,
    top_padding: usize,
    bottom_padding: usize,
    first_y: usize,
    content_bottom: usize,
) {
    let row_bottom = row_top.saturating_add(row_height);
    assert!(first_y >= row_top.saturating_add(top_padding));
    assert!(
        content_bottom.saturating_add(bottom_padding) <= row_bottom,
        "{row_id} ends at {row_bottom}, but content ends at {content_bottom} with {bottom_padding}px bottom padding"
    );
}

fn row_top_for(
    frame: &ChatUiRuntimeFrame,
    message_id: u64,
) -> Result<usize, Box<dyn std::error::Error>> {
    let thread = find_node(frame.tree.root(), "chat.frame.thread").ok_or("missing thread frame")?;
    let mut top = px(&thread.props().common.margin.top)?;
    for id in 1..message_id {
        let row = find_node(frame.tree.root(), &format!("chat.message.{id}.row"))
            .ok_or("missing preceding message row")?;
        top = top.saturating_add(px(&row.props().common.height)?);
    }
    Ok(top)
}

fn assert_thread_content_covers_rows(
    frame: &ChatUiRuntimeFrame,
    message_ids: &[u64],
) -> Result<(), Box<dyn std::error::Error>> {
    let expected_height = message_ids.iter().try_fold(0_usize, |height, id| {
        let row = find_node(frame.tree.root(), &format!("chat.message.{id}.row"))
            .ok_or("missing message row")?;
        Ok::<_, Box<dyn std::error::Error>>(height.saturating_add(px(&row.props().common.height)?))
    })?;
    let thread = find_node(frame.tree.root(), "chat.thread").ok_or("missing thread scroll area")?;
    let required_height = u32::try_from(expected_height)?;

    let expected_height = required_height.max(thread.props().scroll_area.viewport_height);
    assert_eq!(expected_height, thread.props().scroll_area.content_height);
    Ok(())
}

fn assert_second_line_is_rendered(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = document_hits(runtime, frame)
        .into_iter()
        .find(|hit| hit.node_id.as_str() == node_id)
        .ok_or("missing Japanese body hit")?;
    if body.rect.height < 40 {
        return Ok(());
    }
    let y_start = body.rect.y.saturating_add(20);
    let y_end = y_start.saturating_add(20).min(frame.rgba.height as usize);
    let ink = (y_start..y_end).any(|y| {
        (body.rect.x
            ..body
                .rect
                .x
                .saturating_add(body.rect.width)
                .min(frame.rgba.width as usize))
            .any(|x| pixel(frame, x, y) != pixel(frame, 0, 0))
    });
    assert!(ink, "second Japanese text line has no rendered pixels");
    Ok(())
}

fn assert_node_ink_is_visible(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    node_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let node = find_node(frame.tree.root(), node_id).ok_or("missing expanded thinking node")?;
    let hit = hit(&frame.node_hits, node_id).ok_or("missing expanded thinking hit")?;
    let expected = render_node_pixels(runtime, frame, node, hit);
    let glyph_pixels = assert_matching_ink(runtime, frame, &expected);
    assert!(glyph_pixels > 0, "expected rendered thinking text");
    assert!(hit.rect.height >= 40);
    Ok(())
}

fn render_node_pixels(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    node: &katana_ui_core::render_model::UiNode,
    hit: &katana_ui_core::raster_host::UiTreeNodeHit,
) -> Canvas {
    let mut expected = Canvas::new(
        frame.rgba.width as usize,
        frame.rgba.height as usize,
        runtime.canvas_background,
    );
    runtime.renderer.render(
        &mut expected,
        node,
        UiTreeRenderArea {
            x: hit.rect.x,
            y: hit.rect.y,
            width: hit.rect.width,
            height: hit.rect.height,
            scroll_y: 0.0,
        },
    );
    expected
}

fn assert_matching_ink(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    expected: &Canvas,
) -> usize {
    let mut glyph_pixels = 0;
    for (index, color) in expected.pixels().iter().enumerate() {
        if *color != runtime.canvas_background {
            glyph_pixels += 1;
            assert_eq!(
                *color, frame.rgba.pixels[index],
                "content ink clipped at pixel {index}"
            );
        }
    }
    glyph_pixels
}

fn assert_history_row_visible(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let node_id = format!("chat.history.session.{index}");
    let history_row = frame
        .action_hits
        .iter()
        .find(|hit| hit.action.target.as_str() == node_id)
        .ok_or("Storybook history row is outside the visible KUC action hit tree")?;
    let viewport = history_scroll_rect(frame)?;
    assert_action_hit_within(history_row.rect, viewport);
    assert!(history_row.rect.y >= 40);
    assert!(
        history_row.rect.y + history_row.rect.height <= frame.composer.bounds.y as usize,
        "history row {index} ends at {}, composer starts at {}, history viewport {:?}",
        history_row.rect.y + history_row.rect.height,
        frame.composer.bounds.y,
        history_scroll_rect(frame)?
    );
    let has_ink = has_non_background_ink(runtime, frame, history_row.rect);
    assert!(has_ink, "visible history row has no rendered pixels");
    Ok(())
}

fn assert_action_hit_within(action: UiTreeHitRect, viewport: katana_ui_core::render_model::UiRect) {
    let viewport_x = usize::try_from(viewport.x).unwrap_or(0);
    let viewport_y = usize::try_from(viewport.y).unwrap_or(0);
    assert!(action.x >= viewport_x);
    assert!(action.y >= viewport_y);
    assert!(
        action.x.saturating_add(action.width) <= viewport_x.saturating_add(viewport.width as usize)
    );
    assert!(
        action.y.saturating_add(action.height)
            <= viewport_y.saturating_add(viewport.height as usize)
    );
}

fn has_non_background_ink(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    rect: UiTreeHitRect,
) -> bool {
    let left = rect.x.saturating_add(4);
    let right = rect
        .x
        .saturating_add(rect.width)
        .min(frame.rgba.width as usize)
        .saturating_sub(4);
    let top = rect.y.saturating_add(2);
    let bottom = rect
        .y
        .saturating_add(rect.height)
        .min(frame.rgba.height as usize)
        .saturating_sub(2);
    let mut colors = HashMap::new();
    for y in top..bottom {
        for x in left..right {
            *colors.entry(pixel(frame, x, y)).or_insert(0_usize) += 1;
        }
    }
    let Some(background) = colors
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(color, _)| color)
    else {
        return false;
    };
    colors
        .keys()
        .any(|color| color != background && *color != runtime.canvas_background)
}

fn assert_history_viewport_pixels_match(
    first: &ChatUiRuntimeFrame,
    second: &ChatUiRuntimeFrame,
    viewport: katana_ui_core::render_model::UiRect,
) {
    for y in viewport.y..viewport.y.saturating_add_unsigned(viewport.height) {
        for x in viewport.x..viewport.x.saturating_add_unsigned(viewport.width) {
            assert_eq!(
                pixel(first, x as usize, y as usize),
                pixel(second, x as usize, y as usize),
                "history viewport pixel differs at ({x}, {y})"
            );
        }
    }
}

fn history_scroll_rect(
    frame: &ChatUiRuntimeFrame,
) -> Result<katana_ui_core::render_model::UiRect, Box<dyn std::error::Error>> {
    let scroll = find_node(frame.tree.root(), "chat.history.scroll")
        .ok_or("missing KUC history scroll area")?;
    let margin = &scroll.props().common.margin;
    Ok(katana_ui_core::render_model::UiRect::new(
        i32::try_from(px(&margin.left)?)?,
        i32::try_from(px(&margin.top)?)?,
        scroll.props().scroll_area.viewport_width,
        scroll.props().scroll_area.viewport_height,
    ))
}

fn rect_center(rect: katana_ui_core::render_model::UiRect) -> (i32, i32) {
    (
        rect.x + i32::try_from(rect.width / 2).unwrap_or(i32::MAX),
        rect.y + i32::try_from(rect.height / 2).unwrap_or(i32::MAX),
    )
}

fn px(dimension: &UiDimension) -> Result<usize, Box<dyn std::error::Error>> {
    match dimension {
        UiDimension::Px(value) => Ok(usize::from(*value)),
        _ => Err("expected fixed row geometry".into()),
    }
}

fn document_hits(
    runtime: &ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
) -> Vec<katana_ui_core::raster_host::UiTreeNodeHit> {
    runtime.renderer.document_node_hit_rects(
        frame.tree.root(),
        UiTreeRenderArea {
            x: 0,
            y: 0,
            width: frame.rgba.width as usize,
            height: usize::MAX,
            scroll_y: 0.0,
        },
    )
}

fn pixel(frame: &ChatUiRuntimeFrame, x: usize, y: usize) -> u32 {
    frame.rgba.pixels[y * frame.rgba.width as usize + x]
}
