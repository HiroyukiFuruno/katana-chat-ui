mod action_button;
mod composer;
mod composer_controls;
#[cfg(test)]
mod composer_tests;
mod markdown;
mod output_cards;
mod output_handoff;
mod panel;
mod styles;
mod thinking;
mod thinking_indicator;
mod thread;
#[cfg(test)]
mod thread_tests;
mod vendor_control_parts;
mod vendor_controls;

pub use action_button::FloemActionIconButton;
pub use composer::{FloemComposerActions, FloemComposerView};
pub use composer_controls::FloemComposerControlsView;
pub use panel::{FloemChatPanel, FloemPanelSlot, FloemPanelSlotView};
pub use thread::FloemThreadView;
pub use vendor_controls::FloemVendorControlsView;

use floem::prelude::*;
use katana_chat_ui::ChatUiSurface;

#[derive(Clone, Copy)]
pub struct FloemChatActions<
    OnAttach,
    OnRemoveAttachment,
    OnSettings,
    OnSubmit,
    OnStop,
    OnVendorSelect,
    OnControlSelect,
> {
    pub on_attach: OnAttach,
    pub on_remove_attachment: OnRemoveAttachment,
    pub on_settings: OnSettings,
    pub on_submit: OnSubmit,
    pub on_stop: OnStop,
    pub on_vendor_select: OnVendorSelect,
    pub on_control_select: OnControlSelect,
}

impl<OnAttach, OnRemoveAttachment, OnSettings, OnSubmit, OnStop, OnVendorSelect, OnControlSelect>
    FloemChatActions<
        OnAttach,
        OnRemoveAttachment,
        OnSettings,
        OnSubmit,
        OnStop,
        OnVendorSelect,
        OnControlSelect,
    >
{
    pub fn new(
        on_attach: OnAttach,
        on_remove_attachment: OnRemoveAttachment,
        on_settings: OnSettings,
        on_submit: OnSubmit,
        on_stop: OnStop,
        on_vendor_select: OnVendorSelect,
        on_control_select: OnControlSelect,
    ) -> Self {
        Self {
            on_attach,
            on_remove_attachment,
            on_settings,
            on_submit,
            on_stop,
            on_vendor_select,
            on_control_select,
        }
    }
}

pub struct FloemChatView;

impl FloemChatView {
    pub fn render<
        OnAttach,
        OnRemoveAttachment,
        OnSettings,
        OnSubmit,
        OnStop,
        OnVendorSelect,
        OnControlSelect,
    >(
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        actions: FloemChatActions<
            OnAttach,
            OnRemoveAttachment,
            OnSettings,
            OnSubmit,
            OnStop,
            OnVendorSelect,
            OnControlSelect,
        >,
    ) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
        OnRemoveAttachment: Fn(usize) + Copy + 'static,
        OnSettings: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnStop: Fn() + Copy + 'static,
        OnVendorSelect: Fn(String) + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        let panel = FloemChatPanel::new()
            .header(FloemPanelSlot::new(toolbar(surface)))
            .thread(FloemPanelSlot::new(FloemThreadView::render(surface)))
            .composer(FloemPanelSlot::new(FloemComposerView::composer(
                surface,
                draft,
                FloemComposerActions::new(
                    actions.on_attach,
                    actions.on_remove_attachment,
                    actions.on_submit,
                    actions.on_stop,
                    actions.on_vendor_select,
                    actions.on_control_select,
                ),
            )))
            .render();
        stack((
            panel,
            output_handoff::FloemOutputHandoffHover::render(surface),
        ))
        .style(|style| style.size_full())
    }
}

fn toolbar(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
    h_stack((toolbar_identity(surface),)).style(|style| {
        style
            .width_full()
            .gap(styles::PANEL_GAP)
            .items_center()
            .justify_start()
    })
}

fn toolbar_identity(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
    let icon_surface = surface;
    let title_surface = surface;
    h_stack((
        dyn_container(
            move || icon_surface.get().chrome.provider_icon.svg,
            |icon_svg| {
                svg(icon_svg).style(|style| {
                    style
                        .size(styles::ICON_SIZE, styles::ICON_SIZE)
                        .color(styles::COLOR_TEXT)
                })
            },
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
