use super::super::super::{ChatUiRuntime, ChatUiRuntimeFrame};
use super::super::helpers::{VIEWPORT, conversation_surface, find_node, responsive_surface};
use super::{action_hit, node_hit};
use katana_ui_core::raster_host::UiTreeHitRect;
use katana_ui_core::render_model::{UiDimension, UiNode};

#[test]
fn composer_controls_share_a_bottom_edge() -> Result<(), Box<dyn std::error::Error>> {
    let surface = responsive_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let model = node_hit(&frame, "chat.composer.control.0")?;
    let thinking = node_hit(&frame, "chat.composer.control.1")?;
    let permission = node_hit(&frame, "chat.composer.control.2")?;

    let bottom = model.rect.y + model.rect.height;
    assert_eq!(bottom, thinking.rect.y + thinking.rect.height);
    assert_eq!(bottom, permission.rect.y + permission.rect.height);
    Ok(())
}

#[test]
fn header_svg_icon_buttons_have_no_visible_border_rings() -> Result<(), Box<dyn std::error::Error>>
{
    let surface = responsive_surface();
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;

    assert_icon_margin_is_background(&frame, &action_hit(&frame, "NewChat")?.rect);
    assert_icon_margin_is_background(&frame, &action_hit(&frame, "ToggleHistory")?.rect);
    Ok(())
}

#[test]
fn completed_message_bodies_have_symmetric_matching_vertical_margins()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = conversation_surface()?;
    let mut runtime = ChatUiRuntime::light(&surface)?;
    let frame = runtime.render(&surface, VIEWPORT)?;
    let thread = tree_node(&frame, "chat.frame.thread")?;
    let top = dimension_px(&thread.props().common.margin.top)?;
    let user_row = tree_node(&frame, "chat.message.1.row")?;
    let user_height = dimension_px(&user_row.props().common.height)?;
    let user_margins = rendered_body_margins(&frame, 1, top)?;
    let agent_margins = rendered_body_margins(&frame, 2, top + user_height)?;
    assert_eq!(user_margins.0, user_margins.1);
    assert_eq!(agent_margins.0, agent_margins.1);
    assert_eq!(user_margins, agent_margins);
    Ok(())
}

fn rendered_body_margins(
    frame: &ChatUiRuntimeFrame,
    id: u64,
    top: usize,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let prefix = format!("chat.message.{id}");
    assert!(find_node(frame.tree.root(), &format!("{prefix}.status")).is_none());
    let body = node_hit(frame, &format!("{prefix}.body.0"))?;
    let card = tree_node(frame, &prefix)?;
    let row = tree_node(frame, &format!("{prefix}.row"))?;
    let bottom = top + dimension_px(&row.props().common.height)?;
    assert!(body.rect.y >= top && body.rect.y + body.rect.height <= bottom);
    let margins = (body.rect.y - top, bottom - body.rect.y - body.rect.height);
    assert_eq!(vertical_padding(card)?, margins);
    Ok(margins)
}

fn tree_node<'a>(
    frame: &'a ChatUiRuntimeFrame,
    node_id: &str,
) -> Result<&'a UiNode, Box<dyn std::error::Error>> {
    find_node(frame.tree.root(), node_id)
        .ok_or_else(|| std::io::Error::other(format!("missing tree node: {node_id}")))
        .map_err(Into::into)
}

fn vertical_padding(node: &UiNode) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    Ok((
        dimension_px(&node.props().common.padding.top)?,
        dimension_px(&node.props().common.padding.bottom)?,
    ))
}

fn dimension_px(dimension: &UiDimension) -> Result<usize, Box<dyn std::error::Error>> {
    match dimension {
        UiDimension::Px(value) => Ok(usize::from(*value)),
        _ => Err(std::io::Error::other("expected pixel layout dimension").into()),
    }
}

fn assert_icon_margin_is_background(frame: &ChatUiRuntimeFrame, rect: &UiTreeHitRect) {
    let background = pixel(frame, 0, 0);
    for inset in 0..2 {
        let left = rect.x + inset;
        let right = rect.x + rect.width - 1 - inset;
        let top = rect.y + inset;
        let bottom = rect.y + rect.height - 1 - inset;
        for x in left..=right {
            assert_eq!(background, pixel(frame, x, top));
            assert_eq!(background, pixel(frame, x, bottom));
        }
        for y in top..=bottom {
            assert_eq!(background, pixel(frame, left, y));
            assert_eq!(background, pixel(frame, right, y));
        }
    }
}

fn pixel(frame: &ChatUiRuntimeFrame, x: usize, y: usize) -> u32 {
    frame.rgba.pixels[y * frame.rgba.width as usize + x]
}
