#[path = "fixture.rs"]
mod fixture;
#[path = "runtime_support.rs"]
mod support;

use katana_chat_ui::{Attachment, ChatUiAction};
use katana_ui_core::render_model::UiNodeKind;

use fixture::{STANDARD_HEIGHT, STANDARD_WIDTH, attachment_surface};
use support::{
    action_left, assert_action, assert_header_action, composer_rect, frame, has_node_fragment,
    node_rect, runtime, select_vendor,
};

#[test]
fn kuc_frame_places_provider_and_toolbar_before_the_composer() -> Result<(), String> {
    let mut runtime = runtime()?;
    let frame = frame(&mut runtime, STANDARD_WIDTH, STANDARD_HEIGHT)?;
    let (provider_x, provider_y, provider_width) =
        node_rect(&frame, "chat.chrome.provider-selector")?;
    let (title_x, title_y, _) = node_rect(&frame, "chat.chrome.title")?;
    let composer_y = frame.composer.bounds.y as usize;
    if provider_x.saturating_add(provider_width) > title_x
        || provider_y >= composer_y
        || title_y >= composer_y
    {
        return Err("KUC did not place the provider selector in the header".into());
    }
    assert_header_action(&frame, "NewChat", composer_y)?;
    assert_header_action(&frame, "ToggleHistory", composer_y)?;
    if has_node_fragment(&frame, "debug")
        || has_node_fragment(&frame, "settings")
        || has_node_fragment(&frame, "output-handoff")
    {
        return Err("standard KUC frame exposed a debug or settings surface".into());
    }
    Ok(())
}

#[test]
fn kuc_dispatches_typed_toolbar_provider_and_submit_actions() -> Result<(), String> {
    let mut runtime = runtime()?;
    let frame = frame(&mut runtime, STANDARD_WIDTH, STANDARD_HEIGHT)?;
    assert_action(&mut runtime, &frame, "NewChat", ChatUiAction::NewChat)?;
    assert_action(
        &mut runtime,
        &frame,
        "ToggleHistory",
        ChatUiAction::ToggleHistory,
    )?;
    assert_action(&mut runtime, &frame, "Submit", ChatUiAction::Submit)?;
    select_vendor(&mut runtime, &frame)
}

#[test]
fn kuc_frame_keeps_usage_and_composer_bounded_on_resize() -> Result<(), String> {
    let mut runtime = runtime()?;
    let frame = frame(&mut runtime, 320, 240)?;
    let (usage_x, _, usage_width) = node_rect(&frame, "chat.composer.usage")?;
    let send_x = action_left(&frame, "Submit")?;
    let (_, composer_y, composer_width) = composer_rect(&frame);
    if composer_width > 320
        || composer_y.saturating_add(frame.composer.bounds.height as usize) > 240
    {
        return Err("KUC composer escaped the narrow viewport".into());
    }
    if usage_width == 0 || usage_x.saturating_add(usage_width) > send_x {
        return Err("KUC did not place usage next to the primary action".into());
    }
    Ok(())
}

#[test]
fn kuc_frame_keeps_user_message_bounded_to_agent_width() -> Result<(), String> {
    let mut runtime = runtime()?;
    let frame = frame(&mut runtime, STANDARD_WIDTH, 900)?;
    let (user_x, _, user_width) = node_rect(&frame, "chat.message.1.body.0")?;
    let (agent_x, _, agent_width) = node_rect(&frame, "chat.message.2.body.0")?;
    let composer_width = frame.composer.bounds.width as usize;
    if user_x <= agent_x || user_width >= agent_width || agent_width != composer_width {
        return Err("KUC message widths did not retain the shared layout contract".into());
    }
    Ok(())
}

#[test]
fn kuc_frame_renders_structured_messages_and_attachment_intent() -> Result<(), String> {
    let mut runtime = runtime()?;
    let frame = frame(&mut runtime, STANDARD_WIDTH, 900)?;
    assert_markdown_structure(&frame)?;
    let attachment = attachment_surface();
    if !matches!(attachment.composer.attachments.as_slice(), [Attachment::FileResource(file)] if file.uri.starts_with("file://"))
    {
        return Err("KUC surface did not preserve the path-drop attachment intent".into());
    }
    Ok(())
}

fn assert_markdown_structure(
    frame: &katana_chat_ui::kuc::ChatUiRuntimeFrame,
) -> Result<(), String> {
    let html = support::node(frame, "chat.message.2.body.0")?;
    let code = support::node(frame, "chat.message.2.body.1")?;
    let table = support::node(frame, "chat.message.2.body.2")?;
    assert_node_text(html, "<script>alert(1)</script>")?;
    assert_code_node(code)?;
    assert_table_node(table)
}

fn assert_code_node(node: &katana_ui_core::render_model::UiNode) -> Result<(), String> {
    let text = node
        .children()
        .first()
        .ok_or("KUC code block has no text")?;
    if node.kind() != UiNodeKind::Panel || text.props().text.role != "markdown-code-rust" {
        return Err("KUC code fence lost its Markdown structure".into());
    }
    assert_node_text(text, "let x = 1;")?;
    if !text
        .props()
        .text
        .spans
        .iter()
        .any(|span| span.style.monospace)
    {
        return Err("KUC code fence lost monospace text".into());
    }
    Ok(())
}

fn assert_table_node(node: &katana_ui_core::render_model::UiNode) -> Result<(), String> {
    if node.kind() != UiNodeKind::Grid || node.children().len() != 2 {
        return Err("KUC table lost its grid rows".into());
    }
    let header = node.children().first().ok_or("KUC table has no header")?;
    let row = node.children().get(1).ok_or("KUC table has no row")?;
    assert_node_text(header.children().first().ok_or("KUC table lacks A")?, "A")?;
    assert_node_text(header.children().get(1).ok_or("KUC table lacks B")?, "B")?;
    assert_node_text(row.children().first().ok_or("KUC table lacks c")?, "c")?;
    assert_node_text(row.children().get(1).ok_or("KUC table lacks d")?, "d")
}

fn assert_node_text(node: &katana_ui_core::render_model::UiNode, text: &str) -> Result<(), String> {
    if node
        .props()
        .text
        .spans
        .iter()
        .any(|span| span.text.contains(text))
    {
        return Ok(());
    }
    Err(format!("KUC tree node omitted text: {text}"))
}
