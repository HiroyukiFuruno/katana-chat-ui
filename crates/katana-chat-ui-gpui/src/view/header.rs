use gpui::{Div, div, prelude::*};
use katana_chat_ui::{ChatUiActionButtonSurface, ChatUiSurface};

use super::styles::GpuiStyles;

const PROVIDER_DROPDOWN_CHEVRON: &str = "⌄";

pub(super) struct GpuiHeaderView;

impl GpuiHeaderView {
    pub(super) fn render(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .items_center()
            .gap(GpuiStyles::px(GpuiStyles::LAYOUT.panel_gap))
            .child(Self::identity(surface))
            .child(div().flex_1())
            .child(Self::toolbar_button(&surface.chrome.new_chat))
            .child(Self::toolbar_button(&surface.chrome.history))
    }

    fn identity(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .items_center()
            .gap(GpuiStyles::px(GpuiStyles::LAYOUT.panel_gap))
            .child(Self::provider_icon(surface))
            .child(Self::provider_selector(surface))
            .child(surface.chrome.title.clone())
    }

    fn provider_icon(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .child(GpuiStyles::provider_symbol(
                &surface.chrome.provider_icon.asset_id,
            ))
    }

    fn provider_selector(surface: &ChatUiSurface) -> Div {
        let chevron = if surface.vendor_bar.vendor_options.len() > 1 {
            PROVIDER_DROPDOWN_CHEVRON
        } else {
            ""
        };
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(surface.vendor_bar.active_vendor_label.clone())
            .child(chevron)
    }

    fn toolbar_button(action: &ChatUiActionButtonSurface) -> Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(GpuiStyles::px(GpuiStyles::TOOLBAR_BUTTON_SIZE))
            .h(GpuiStyles::px(GpuiStyles::TOOLBAR_BUTTON_SIZE))
            .child(GpuiStyles::action_symbol(&action.icon.asset_id))
    }
}
