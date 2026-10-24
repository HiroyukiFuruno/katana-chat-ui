use super::{ChatUiActionError, chrome, composer, content, node};
use crate::ChatUiSurface;
use katana_ui_core::render_model::{
    UiCommonProps, UiDisplay, UiLayoutAxis, UiNode, UiNodeKind, UiTree,
};

mod frame;
pub(crate) use frame::composer_input_bounds;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatUiTree {
    source: ChatUiSurface,
    tree: UiTree,
}

impl ChatUiTree {
    pub fn source(&self) -> &ChatUiSurface {
        &self.source
    }

    pub fn tree(&self) -> &UiTree {
        &self.tree
    }
}

pub struct ChatUiTreeBuilder;

pub(crate) struct ChatUiTreeViewport<'a> {
    pub(crate) scroll_y: u32,
    pub(crate) hide_thread: bool,
    pub(crate) history_scroll_y: u32,
    pub(crate) history_content_height: u32,
    pub(crate) history_row_heights: &'a [u16],
    pub(crate) permission_scroll_y: u32,
    pub(crate) permission_content_height: u32,
    pub(crate) message_heights: &'a [u16],
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl ChatUiTreeBuilder {
    pub fn build(surface: &ChatUiSurface) -> Result<ChatUiTree, ChatUiActionError> {
        Self::build_with_open_dropdown(surface, None)
    }

    pub(crate) fn build_with_open_dropdown(
        surface: &ChatUiSurface,
        open_dropdown: Option<&str>,
    ) -> Result<ChatUiTree, ChatUiActionError> {
        let root = root(surface, open_dropdown)?;
        Ok(ChatUiTree {
            source: surface.clone(),
            tree: UiTree::new(root),
        })
    }

    pub(crate) fn build_for_viewport(
        surface: &ChatUiSurface,
        open_dropdown: Option<&str>,
        viewport: ChatUiTreeViewport<'_>,
    ) -> Result<ChatUiTree, ChatUiActionError> {
        frame::build(surface, open_dropdown, viewport)
    }
}

fn root(surface: &ChatUiSurface, open_dropdown: Option<&str>) -> Result<UiNode, ChatUiActionError> {
    let root = node::container(UiNodeKind::Panel, "Chat UI", "chat.root")
        .common(
            UiCommonProps::default()
                .display(UiDisplay::Flex)
                .layout_axis(UiLayoutAxis::Vertical),
        )
        .child(chrome::chrome(
            &surface.chrome,
            &surface.vendor_bar,
            open_dropdown,
            super::composer::DEFAULT_COMPOSER_WIDTH,
        )?)
        .child(content::thread(&surface.message_list.messages)?)
        .child(composer::composer(
            &surface.composer,
            &surface.vendor_bar,
            &surface.usage,
            open_dropdown,
            super::composer::DEFAULT_COMPOSER_WIDTH,
        )?);
    optional_root_children(root, surface)
}

fn optional_root_children(
    mut root: UiNode,
    surface: &ChatUiSurface,
) -> Result<UiNode, ChatUiActionError> {
    if surface.history_panel.visible {
        root = root.child(chrome::history(&surface.history_panel, &[])?);
    }
    if !surface.output_handoff.outputs.is_empty() {
        root = root.child(output_handoff(surface)?);
    }
    if surface.vendor_bar.permission_request.is_some() {
        root = root.child(super::permission::panel(
            &surface.vendor_bar,
            0,
            super::composer::DEFAULT_COMPOSER_WIDTH,
            128,
            54,
            54,
        )?);
    }
    Ok(root)
}

pub(super) fn output_handoff(
    surface: &ChatUiSurface,
) -> Result<katana_ui_core::render_model::UiNode, ChatUiActionError> {
    let mut handoff = node::container(
        UiNodeKind::Panel,
        &surface.output_handoff.label,
        "chat.output-handoff",
    );
    for (index, output) in surface.output_handoff.outputs.iter().enumerate() {
        handoff = handoff.child(super::output::output_node(
            output,
            &format!("chat.output-handoff.output.{index}"),
        )?);
    }
    Ok(handoff)
}

#[cfg(test)]
#[path = "tree/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tree/visibility_tests.rs"]
mod visibility_tests;
