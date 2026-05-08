use gpui::{Div, div, prelude::*};
use katana_chat_ui::{
    Attachment, ChatUiActionButtonSurface, ChatUiComposerSurface, ChatUiSurface,
    ChatUiVendorControlSurface,
};

use super::styles::GpuiStyles;

pub(super) struct GpuiComposerView;

impl GpuiComposerView {
    pub(super) fn render(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .flex_col()
            .justify_between()
            .w_full()
            .max_w(GpuiStyles::px(GpuiStyles::LAYOUT.chat_body_max_width))
            .rounded_lg()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.border))
            .bg(GpuiStyles::color(GpuiStyles::COLORS.assistant))
            .p(GpuiStyles::px(GpuiStyles::LAYOUT.composer_padding))
            .h(Self::composer_height())
            .gap(GpuiStyles::px(GpuiStyles::LAYOUT.panel_gap))
            .child(Self::input_area(&surface.composer))
            .child(Self::attachment_tray(&surface.composer))
            .child(Self::controls(surface))
    }

    fn composer_height() -> gpui::Pixels {
        GpuiStyles::px(
            GpuiStyles::LAYOUT.composer_input_height
                + (GpuiStyles::LAYOUT.composer_padding * 2.0)
                + 64.0,
        )
    }

    fn input_area(composer: &ChatUiComposerSurface) -> Div {
        div()
            .flex()
            .w_full()
            .min_h(GpuiStyles::px(GpuiStyles::LAYOUT.composer_input_height))
            .text_color(GpuiStyles::color(GpuiStyles::COLORS.muted))
            .child(Self::input_text(composer))
    }

    fn input_text(composer: &ChatUiComposerSurface) -> String {
        if composer.text.is_empty() {
            return composer.placeholder.clone();
        }
        composer.text.clone()
    }

    fn attachment_tray(composer: &ChatUiComposerSurface) -> Div {
        div()
            .flex()
            .items_center()
            .gap(GpuiStyles::px(GpuiStyles::CONTROL_ROW_GAP))
            .children(composer.attachments.iter().map(Self::attachment_chip))
    }

    fn attachment_chip(attachment: &Attachment) -> Div {
        div()
            .rounded_md()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.border))
            .px_3()
            .py_2()
            .child(Self::attachment_label(attachment))
    }

    fn attachment_label(attachment: &Attachment) -> String {
        match attachment {
            Attachment::Text(text) => text.label.clone(),
            Attachment::FileResource(file) => file.uri.clone(),
            Attachment::ImageResource(image) => image.mime_type.clone(),
            Attachment::Unsupported(unsupported) => unsupported.label.clone(),
        }
    }

    fn controls(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .items_center()
            .gap(GpuiStyles::px(GpuiStyles::CONTROL_ROW_GAP))
            .child(Self::action_button(&surface.composer.attach))
            .child(div().flex_1())
            .child(Self::vendor_controls(surface))
            .child(div().flex_1())
            .child(Self::usage_meter(surface))
            .child(Self::primary_action(&surface.composer))
    }

    fn vendor_controls(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .gap(GpuiStyles::px(GpuiStyles::CONTROL_ROW_GAP))
            .children(surface.vendor_bar.controls.iter().map(Self::control))
    }

    fn control(control: &ChatUiVendorControlSurface) -> Div {
        div()
            .rounded_md()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.border))
            .px_3()
            .py_2()
            .child(format!("{}: {}  ⌄", control.label, control.value))
    }

    fn usage_meter(surface: &ChatUiSurface) -> Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(GpuiStyles::px(GpuiStyles::ICON_BUTTON_SIZE))
            .h(GpuiStyles::px(GpuiStyles::ICON_BUTTON_SIZE))
            .rounded_full()
            .border_1()
            .border_color(GpuiStyles::color(GpuiStyles::COLORS.text))
            .child(format!("{}%", surface.usage.context_percentage))
    }

    fn primary_action(composer: &ChatUiComposerSurface) -> Div {
        if composer.stop_enabled {
            return Self::action_button(&composer.stop);
        }
        Self::action_button(&composer.send)
    }

    fn action_button(action: &ChatUiActionButtonSurface) -> Div {
        GpuiStyles::icon_button(GpuiStyles::action_symbol(&action.icon.asset_id))
    }
}
