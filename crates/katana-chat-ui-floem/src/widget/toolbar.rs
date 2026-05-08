use floem::prelude::*;
use katana_chat_ui::{ChatUiSurface, SvgIcon};

use super::{FloemActionIconButton, provider_icon_selector::FloemProviderIconSelector, styles};

pub(super) struct FloemToolbar;

impl FloemToolbar {
    pub(super) fn render<OnNewChat, OnHistory, OnSettings, OnVendorSelect>(
        surface: RwSignal<ChatUiSurface>,
        on_new_chat: OnNewChat,
        on_history: OnHistory,
        on_settings: OnSettings,
        on_vendor_select: OnVendorSelect,
    ) -> impl IntoView
    where
        OnNewChat: Fn() + Copy + 'static,
        OnHistory: Fn() + Copy + 'static,
        OnSettings: Fn() + Copy + 'static,
        OnVendorSelect: Fn(String) + Copy + 'static,
    {
        h_stack((
            toolbar_identity(surface, on_vendor_select),
            toolbar_actions(surface, on_new_chat, on_history, on_settings),
        ))
        .style(|style| {
            style
                .width_full()
                .gap(styles::PANEL_GAP)
                .items_center()
                .justify_between()
        })
    }
}

fn toolbar_identity<OnVendorSelect>(
    surface: RwSignal<ChatUiSurface>,
    on_vendor_select: OnVendorSelect,
) -> impl IntoView
where
    OnVendorSelect: Fn(String) + Copy + 'static,
{
    let vendor_surface = surface;
    let title_surface = surface;
    h_stack((
        dyn_container(move || surface.get().chrome.provider_icon, provider_icon),
        dyn_container(
            move || {
                let surface = vendor_surface.get();
                surface.vendor_bar
            },
            move |vendor| FloemProviderIconSelector::render(vendor, on_vendor_select),
        ),
        label(move || title_surface.get().chrome.title)
            .style(styles::FloemWidgetStyle::toolbar_title),
    ))
    .style(|style| {
        style
            .min_width(0.0)
            .gap(styles::PANEL_GAP)
            .items_center()
            .justify_start()
            .flex_grow(1.0)
            .flex_shrink(1.0)
    })
}

fn provider_icon(icon: SvgIcon) -> impl IntoView {
    svg(icon.svg).style(|style| {
        style
            .size(styles::ICON_SIZE, styles::ICON_SIZE)
            .color(styles::COLOR_TEXT)
            .flex_shrink(0.0)
    })
}

fn toolbar_actions<OnNewChat, OnHistory, OnSettings>(
    surface: RwSignal<ChatUiSurface>,
    on_new_chat: OnNewChat,
    on_history: OnHistory,
    on_settings: OnSettings,
) -> impl IntoView
where
    OnNewChat: Fn() + Copy + 'static,
    OnHistory: Fn() + Copy + 'static,
    OnSettings: Fn() + Copy + 'static,
{
    h_stack((
        dyn_container(
            move || surface.get().chrome.new_chat.clone(),
            move |action| FloemActionIconButton::render(action, on_new_chat),
        ),
        dyn_container(
            move || surface.get().chrome.history.clone(),
            move |action| FloemActionIconButton::render(action, on_history),
        ),
        dyn_container(
            move || surface.get().chrome.settings.clone(),
            move |action| FloemActionIconButton::render(action, on_settings),
        ),
    ))
    .style(|style| {
        style
            .gap(styles::PANEL_GAP)
            .items_center()
            .justify_end()
            .flex_shrink(0.0)
    })
}
