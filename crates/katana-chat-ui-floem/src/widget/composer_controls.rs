use super::{
    FloemActionIconButton, FloemVendorControlsView,
    composer::{ComposerDraftSubmitter, ComposerSubmitGate},
    styles,
};
use floem::{AnyView, prelude::*, taffy::style::FlexWrap};
use katana_chat_ui::{ChatUiActionButtonSurface, ChatUiComposerSurface, ChatUiVendorBarSurface};

pub struct FloemComposerControlsView;

#[derive(Clone)]
pub struct FloemComposerControlActions<OnAttach, OnStop, OnSubmit, OnVendorSelect, OnControlSelect>
{
    on_attach: OnAttach,
    on_stop: OnStop,
    on_submit: OnSubmit,
    on_vendor_select: OnVendorSelect,
    on_control_select: OnControlSelect,
}

impl<OnAttach, OnStop, OnSubmit, OnVendorSelect, OnControlSelect>
    FloemComposerControlActions<OnAttach, OnStop, OnSubmit, OnVendorSelect, OnControlSelect>
{
    pub fn new(
        on_attach: OnAttach,
        on_stop: OnStop,
        on_submit: OnSubmit,
        on_vendor_select: OnVendorSelect,
        on_control_select: OnControlSelect,
    ) -> Self {
        Self {
            on_attach,
            on_stop,
            on_submit,
            on_vendor_select,
            on_control_select,
        }
    }
}

impl FloemComposerControlsView {
    pub fn render<OnAttach, OnStop, OnSubmit, OnVendorSelect, OnControlSelect>(
        composer: ChatUiComposerSurface,
        vendor: ChatUiVendorBarSurface,
        draft: RwSignal<String>,
        actions: FloemComposerControlActions<
            OnAttach,
            OnStop,
            OnSubmit,
            OnVendorSelect,
            OnControlSelect,
        >,
    ) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
        OnStop: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnVendorSelect: Fn(String) + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        let attach = composer.attach.clone();
        let on_submit = actions.on_submit.clone();
        h_stack((
            Self::left(attach, actions.on_attach),
            FloemVendorControlsView::render_header(
                vendor,
                actions.on_vendor_select,
                actions.on_control_select,
            ),
            Self::right(composer, draft, actions.on_stop, on_submit),
        ))
        .style(|style| {
            style
                .width_full()
                .gap(styles::PANEL_GAP)
                .flex_wrap(FlexWrap::Wrap)
                .justify_between()
                .items_center()
        })
    }

    fn left<OnAttach>(attach: ChatUiActionButtonSurface, on_attach: OnAttach) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
    {
        h_stack((FloemActionIconButton::render(attach, on_attach),))
            .style(|style| style.gap(styles::PANEL_GAP).items_center())
    }

    fn right<OnStop, OnSubmit>(
        composer: ChatUiComposerSurface,
        draft: RwSignal<String>,
        on_stop: OnStop,
        on_submit: OnSubmit,
    ) -> impl IntoView
    where
        OnStop: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
    {
        h_stack((Self::primary_action(composer, draft, on_stop, on_submit),))
            .style(|style| style.gap(styles::PANEL_GAP).items_center())
    }

    fn primary_action<OnStop, OnSubmit>(
        composer: ChatUiComposerSurface,
        draft: RwSignal<String>,
        on_stop: OnStop,
        on_submit: OnSubmit,
    ) -> AnyView
    where
        OnStop: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
    {
        if composer.stop_enabled {
            return FloemActionIconButton::render(composer.stop, on_stop).into_any();
        }
        let composer_for_gate = composer.clone();
        let composer_for_submit = composer.clone();
        FloemActionIconButton::render_with_enabled(
            composer.send,
            move || ComposerSubmitGate::can_submit(&composer_for_gate, &draft.get()),
            move || {
                ComposerDraftSubmitter::submit_allowed(
                    &composer_for_submit,
                    draft,
                    on_submit.clone(),
                )
            },
        )
        .into_any()
    }
}
