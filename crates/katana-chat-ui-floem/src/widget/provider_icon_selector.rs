use super::vendor_control_parts::VendorControlParts;
use floem::{AnyView, menu::Menu, peniko::Color, prelude::*};
use katana_chat_ui::ChatUiVendorBarSurface;

const BUTTON_SIZE: f64 = 34.0;
const BUTTON_RADIUS: f64 = 12.0;
const CHEVRON_SIZE: f64 = 14.0;
const BUTTON_GAP: f64 = 6.0;
const BUTTON_PADDING_X: f64 = 8.0;
const CHEVRON_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" data-kcu-icon="provider-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>"#;

pub(super) struct FloemProviderIconSelector;

impl FloemProviderIconSelector {
    pub(super) fn render<OnVendorSelect>(
        vendor: ChatUiVendorBarSurface,
        on_vendor_select: OnVendorSelect,
    ) -> AnyView
    where
        OnVendorSelect: Fn(String) + Copy + 'static,
    {
        let options = VendorControlParts::vendor_options(&vendor.vendor_options);
        let multiple_options = options.len() > 1;
        let button = provider_dropdown_button(vendor.active_vendor_label.clone(), multiple_options);
        if options.len() <= 1 {
            return button
                .tooltip({
                    let label = vendor.active_vendor_label.clone();
                    move || VendorControlParts::tooltip_label(label.clone())
                })
                .into_any();
        }
        button
            .popout_menu(move || vendor_menu(options.clone(), on_vendor_select))
            .tooltip({
                let label = vendor.vendor_selector_label.clone();
                move || VendorControlParts::tooltip_label(label.clone())
            })
            .into_any()
    }
}

fn provider_dropdown_button(active_label: String, multiple_options: bool) -> impl IntoView {
    container(h_stack((
        label(move || active_label.clone())
            .style(super::styles::FloemWidgetStyle::provider_selector_label),
        dropdown_hint(multiple_options),
    )))
    .style(|style| {
        style
            .height(BUTTON_SIZE)
            .padding_horiz(BUTTON_PADDING_X)
            .gap(BUTTON_GAP)
            .items_center()
            .justify_center()
            .border_radius(BUTTON_RADIUS)
            .background(Color::TRANSPARENT)
            .flex_shrink(0.0)
    })
}

fn dropdown_hint(visible: bool) -> impl IntoView {
    svg(CHEVRON_ICON).style(move |style| {
        style
            .size(CHEVRON_SIZE, CHEVRON_SIZE)
            .color(if visible {
                super::styles::COLOR_TEXT
            } else {
                Color::TRANSPARENT
            })
            .flex_shrink(0.0)
    })
}

fn vendor_menu<OnVendorSelect>(
    options: Vec<super::vendor_control_parts::VendorChoice>,
    on_vendor_select: OnVendorSelect,
) -> Menu
where
    OnVendorSelect: Fn(String) + Copy + 'static,
{
    options.into_iter().fold(Menu::new(), move |menu, choice| {
        let id = choice.id;
        menu.item(choice.label, move |item| {
            item.action(move || on_vendor_select(id.clone()))
        })
    })
}
