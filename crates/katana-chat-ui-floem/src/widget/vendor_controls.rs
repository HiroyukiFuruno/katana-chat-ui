use super::vendor_control_parts::VendorControlParts;
use floem::{AnyView, menu::Menu, prelude::*, style::FlexWrap};
use katana_chat_ui::{ChatUiVendorBarSurface, ChatUiVendorControlSurface};

const CONTROL_ROW_GAP: f64 = 10.0;

pub struct FloemVendorControlsView;

impl FloemVendorControlsView {
    pub fn render(vendor: ChatUiVendorBarSurface) -> impl IntoView {
        Self::render_header(vendor, |_| {}, |_, _| {})
    }

    pub fn render_header<OnVendorSelect, OnControlSelect>(
        vendor: ChatUiVendorBarSurface,
        _on_vendor_select: OnVendorSelect,
        on_control_select: OnControlSelect,
    ) -> impl IntoView
    where
        OnVendorSelect: Fn(String) + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        Self::render_controls(vendor, on_control_select)
    }

    pub(super) fn render_controls<OnControlSelect>(
        vendor: ChatUiVendorBarSurface,
        on_control_select: OnControlSelect,
    ) -> impl IntoView
    where
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        v_stack((dyn_stack(
            move || VendorControlParts::header_controls(&vendor.controls),
            |control| control.key.clone(),
            move |control| Self::control(control, on_control_select),
        )
        .style(|style| {
            style
                .gap(CONTROL_ROW_GAP)
                .flex_wrap(FlexWrap::Wrap)
                .items_center()
                .justify_center()
                .min_width(0.0)
                .flex_shrink(1.0)
        }),))
        .style(|style| {
            style
                .gap(CONTROL_ROW_GAP)
                .items_center()
                .justify_center()
                .min_width(0.0)
                .flex_shrink(1.0)
        })
    }

    fn control<OnControlSelect>(
        control: ChatUiVendorControlSurface,
        on_control_select: OnControlSelect,
    ) -> AnyView
    where
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        if control.options.is_empty() {
            return Self::static_control(control);
        }
        Self::dropdown_control(control, on_control_select)
    }

    fn static_control(control: ChatUiVendorControlSurface) -> AnyView {
        VendorControlParts::static_chip(VendorControlParts::control_text(&control), control.enabled)
            .tooltip({
                let label = control.label.clone();
                move || VendorControlParts::tooltip_label(label.clone())
            })
            .into_any()
    }

    fn dropdown_control<OnControlSelect>(
        control: ChatUiVendorControlSurface,
        on_control_select: OnControlSelect,
    ) -> AnyView
    where
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        let key = control.key.clone();
        let tooltip_label = control.label.clone();
        let options = control
            .options
            .iter()
            .map(|value| VendorControlParts::control_choice(value))
            .collect::<Vec<_>>();
        VendorControlParts::selector_chip(
            VendorControlParts::control_text(&control),
            control.enabled,
        )
        .popout_menu(move || {
            control_menu(
                key.clone(),
                options.clone(),
                control.enabled,
                on_control_select,
            )
        })
        .tooltip(move || VendorControlParts::tooltip_label(tooltip_label.clone()))
        .into_any()
    }
}

fn control_menu<OnControlSelect>(
    key: String,
    options: Vec<super::vendor_control_parts::ControlChoice>,
    enabled: bool,
    on_control_select: OnControlSelect,
) -> Menu
where
    OnControlSelect: Fn(String, String) + Copy + 'static,
{
    options.into_iter().fold(Menu::new(), move |menu, choice| {
        let key = key.clone();
        let value = choice.value;
        menu.item(choice.label, move |item| {
            item.enabled(enabled)
                .action(move || on_control_select(key.clone(), value.clone()))
        })
    })
}
