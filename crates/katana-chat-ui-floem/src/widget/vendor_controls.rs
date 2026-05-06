use super::{styles, vendor_control_parts::VendorControlParts};
use floem::{
    AnyView,
    menu::{Menu, MenuItem},
    prelude::*,
    taffy::style::FlexWrap,
};
use katana_chat_ui::{ChatUiVendorBarSurface, ChatUiVendorControlSurface};

pub struct FloemVendorControlsView;

impl FloemVendorControlsView {
    pub fn render(vendor: ChatUiVendorBarSurface) -> impl IntoView {
        Self::render_header(vendor, |_| {}, |_, _| {})
    }

    pub fn render_header<OnVendorSelect, OnControlSelect>(
        vendor: ChatUiVendorBarSurface,
        on_vendor_select: OnVendorSelect,
        on_control_select: OnControlSelect,
    ) -> impl IntoView
    where
        OnVendorSelect: Fn(String) + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        h_stack((
            Self::vendor_dropdown(&vendor, on_vendor_select),
            dyn_stack(
                move || VendorControlParts::header_controls(&vendor.controls),
                |control| control.key.clone(),
                move |control| Self::control(control, on_control_select),
            )
            .style(|style| {
                style
                    .gap(styles::PANEL_GAP)
                    .flex_wrap(FlexWrap::Wrap)
                    .items_center()
            }),
        ))
        .style(|style| {
            style
                .gap(styles::PANEL_GAP)
                .flex_wrap(FlexWrap::Wrap)
                .items_center()
        })
    }

    fn vendor_dropdown<OnVendorSelect>(
        vendor: &ChatUiVendorBarSurface,
        on_vendor_select: OnVendorSelect,
    ) -> AnyView
    where
        OnVendorSelect: Fn(String) + Copy + 'static,
    {
        let current = super::vendor_control_parts::VendorChoice {
            id: vendor.active_vendor_id.clone(),
            label: vendor.active_vendor_label.clone(),
        };
        let options = VendorControlParts::vendor_options(&vendor.vendor_options);
        if options.is_empty() {
            return empty_vendor_dropdown(current.label, vendor.vendor_selector_label.clone());
        }
        if options.len() == 1 {
            return VendorControlParts::static_chip(current.label, true)
                .tooltip({
                    let label = vendor.vendor_selector_label.clone();
                    move || VendorControlParts::tooltip_label(label.clone())
                })
                .into_any();
        }
        VendorControlParts::selector_chip(current.label, true)
            .popout_menu(move || vendor_menu(options.clone(), on_vendor_select))
            .tooltip({
                let label = vendor.vendor_selector_label.clone();
                move || VendorControlParts::tooltip_label(label.clone())
            })
            .into_any()
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
        let menu_label = control.label.clone();
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
                menu_label.clone(),
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

fn empty_vendor_dropdown(label: String, tooltip: String) -> AnyView {
    if label.is_empty() {
        return empty().into_any();
    }
    VendorControlParts::static_chip(label, false)
        .tooltip(move || VendorControlParts::tooltip_label(tooltip.clone()))
        .into_any()
}

fn vendor_menu<OnVendorSelect>(
    options: Vec<super::vendor_control_parts::VendorChoice>,
    on_vendor_select: OnVendorSelect,
) -> Menu
where
    OnVendorSelect: Fn(String) + Copy + 'static,
{
    options
        .into_iter()
        .fold(Menu::new("Vendor"), move |menu, choice| {
            let id = choice.id;
            menu.entry(MenuItem::new(choice.label).action(move || on_vendor_select(id.clone())))
        })
}

fn control_menu<OnControlSelect>(
    label: String,
    key: String,
    options: Vec<super::vendor_control_parts::ControlChoice>,
    enabled: bool,
    on_control_select: OnControlSelect,
) -> Menu
where
    OnControlSelect: Fn(String, String) + Copy + 'static,
{
    options
        .into_iter()
        .fold(Menu::new(label), move |menu, choice| {
            let key = key.clone();
            let value = choice.value;
            menu.entry(
                MenuItem::new(choice.label)
                    .enabled(enabled)
                    .action(move || on_control_select(key.clone(), value.clone())),
            )
        })
}
