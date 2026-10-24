use super::super::super::{ChatUiActionError, composer, node, vendor};
use super::{ChatFrameLayout, HEADER_HEIGHT};
use crate::ChatUiSurface;
use katana_ui_core::render_model::{
    UiCommonProps, UiDimension, UiEdgeInsets, UiNode, UiNodeKind, UiPosition,
};

const PROVIDER_SELECTOR: &str = "chat.chrome.provider-selector";
const CONTROL_PREFIX: &str = "chat.composer.control.";
const CONTROL_GAP: u16 = 8;

pub(super) fn append(
    root: UiNode,
    surface: &ChatUiSurface,
    open_dropdown: Option<&str>,
    layout: ChatFrameLayout,
) -> Result<UiNode, ChatUiActionError> {
    let Some(dropdown_id) = open_dropdown else {
        return Ok(root);
    };
    let Some(overlay) = overlay(surface, dropdown_id, layout)? else {
        return Ok(root);
    };
    Ok(root.child(overlay))
}

fn overlay(
    surface: &ChatUiSurface,
    dropdown_id: &str,
    layout: ChatFrameLayout,
) -> Result<Option<UiNode>, ChatUiActionError> {
    let width = vendor::control_width(layout.body_width);
    let options = vendor::options(&surface.vendor_bar, dropdown_id, width)?;
    if options.is_empty() {
        return Ok(None);
    }
    let height = menu_height(options.len());
    let placement = placement(dropdown_id, surface, layout, height, width);
    Ok(Some(menu(options, placement, height, width)))
}

fn menu(options: Vec<UiNode>, placement: MenuPlacement, height: u16, width: u16) -> UiNode {
    let mut menu = node::container(UiNodeKind::Column, "", "chat.dropdown.menu").common(
        UiCommonProps::default()
            .width(UiDimension::px(width))
            .height(UiDimension::px(height))
            .position(UiPosition::Absolute)
            .margin(placement.margin()),
    );
    for option in options {
        menu = menu.child(option);
    }
    menu
}

fn placement(
    dropdown_id: &str,
    surface: &ChatUiSurface,
    layout: ChatFrameLayout,
    height: u16,
    width: u16,
) -> MenuPlacement {
    let (left, anchor_top) = anchor(dropdown_id, surface, layout, width);
    let top = if dropdown_id == PROVIDER_SELECTOR {
        menu_top(anchor_top, height, layout.viewport_height)
    } else {
        composer_menu_top(layout, height)
    };
    MenuPlacement { left, top }
}

fn composer_menu_top(layout: ChatFrameLayout, height: u16) -> u16 {
    layout
        .composer_top()
        .saturating_add(composer::INPUT_HEIGHT)
        .saturating_sub(height)
}

fn anchor(
    dropdown_id: &str,
    surface: &ChatUiSurface,
    layout: ChatFrameLayout,
    width: u16,
) -> (u16, u16) {
    if dropdown_id == PROVIDER_SELECTOR {
        return (layout.side_margin, HEADER_HEIGHT);
    }
    let Some(index) = control_index(dropdown_id) else {
        return (layout.side_margin, HEADER_HEIGHT);
    };
    let stacked =
        vendor::controls_are_stacked(surface.vendor_bar.controls.len(), layout.body_width);
    let step = width.saturating_add(CONTROL_GAP);
    let offset = u16::try_from(index).map_or(u16::MAX, |value| value.saturating_mul(step));
    let left = if stacked {
        layout.side_margin
    } else {
        layout.side_margin.saturating_add(offset)
    };
    let top = layout.composer_top();
    (left, top)
}

fn menu_top(anchor_top: u16, height: u16, viewport_height: u16) -> u16 {
    if anchor_top.saturating_add(height) <= viewport_height {
        return anchor_top;
    }
    anchor_top.saturating_sub(height)
}

fn menu_height(option_count: usize) -> u16 {
    u16::try_from(option_count).map_or(u16::MAX, |count| count.saturating_mul(vendor::OPTION_STEP))
}

fn control_index(dropdown_id: &str) -> Option<usize> {
    dropdown_id
        .strip_prefix(CONTROL_PREFIX)?
        .parse::<usize>()
        .ok()
}

struct MenuPlacement {
    left: u16,
    top: u16,
}

impl MenuPlacement {
    fn margin(&self) -> UiEdgeInsets {
        UiEdgeInsets {
            top: UiDimension::px(self.top),
            right: UiDimension::px(0),
            bottom: UiDimension::px(0),
            left: UiDimension::px(self.left),
        }
    }
}
