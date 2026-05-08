use super::{
    FloemActionIconButton, FloemVendorControlsView,
    composer::{ComposerDraftSubmitter, ComposerSubmitGate},
    composer_usage::FloemComposerUsageView,
};
use floem::{AnyView, prelude::*};
use katana_chat_ui::{
    ChatUiActionButtonSurface, ChatUiComposerSurface, ChatUiUsageSurface, ChatUiVendorBarSurface,
};

const CONTROL_ROW_GAP: f64 = 10.0;

pub struct FloemComposerControlsView;

#[derive(Clone)]
pub struct FloemComposerControlActions<OnAttach, OnStop, OnSubmit, OnControlSelect> {
    on_attach: OnAttach,
    on_stop: OnStop,
    on_submit: OnSubmit,
    on_control_select: OnControlSelect,
}

impl<OnAttach, OnStop, OnSubmit, OnControlSelect>
    FloemComposerControlActions<OnAttach, OnStop, OnSubmit, OnControlSelect>
{
    pub fn new(
        on_attach: OnAttach,
        on_stop: OnStop,
        on_submit: OnSubmit,
        on_control_select: OnControlSelect,
    ) -> Self {
        Self {
            on_attach,
            on_stop,
            on_submit,
            on_control_select,
        }
    }
}

struct ComposerControlSurfaces {
    attach: ChatUiActionButtonSurface,
    usage: ChatUiUsageSurface,
    vendor: ChatUiVendorBarSurface,
    composer: ChatUiComposerSurface,
}

struct ComposerFooterActions<OnAttach, OnStop, OnSubmit, OnControlSelect> {
    on_attach: OnAttach,
    on_control_select: OnControlSelect,
    on_stop: OnStop,
    on_submit: OnSubmit,
}

impl FloemComposerControlsView {
    pub fn render<OnAttach, OnStop, OnSubmit, OnControlSelect>(
        composer: ChatUiComposerSurface,
        usage: ChatUiUsageSurface,
        vendor: ChatUiVendorBarSurface,
        draft: RwSignal<String>,
        actions: FloemComposerControlActions<OnAttach, OnStop, OnSubmit, OnControlSelect>,
    ) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
        OnStop: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        let surfaces = ComposerControlSurfaces {
            attach: composer.attach.clone(),
            vendor,
            usage,
            composer,
        };
        let footer_actions = ComposerFooterActions {
            on_attach: actions.on_attach,
            on_control_select: actions.on_control_select,
            on_stop: actions.on_stop,
            on_submit: actions.on_submit.clone(),
        };
        Self::action_row(draft, surfaces, footer_actions)
            .style(|style| style.width_full().min_width(0.0))
    }

    fn left<OnAttach>(attach: ChatUiActionButtonSurface, on_attach: OnAttach) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
    {
        h_stack((FloemActionIconButton::render(attach, on_attach),))
            .style(|style| style.gap(CONTROL_ROW_GAP).items_center().flex_shrink(0.0))
    }

    fn center<OnControlSelect>(
        vendor: ChatUiVendorBarSurface,
        on_control_select: OnControlSelect,
    ) -> impl IntoView
    where
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        h_stack((FloemVendorControlsView::render_header(
            vendor,
            |_| {},
            on_control_select,
        ),))
        .style(|style| {
            style
                .min_width(0.0)
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .justify_center()
                .items_center()
        })
    }

    fn action_row<OnAttach, OnStop, OnSubmit, OnControlSelect>(
        draft: RwSignal<String>,
        surfaces: ComposerControlSurfaces,
        actions: ComposerFooterActions<OnAttach, OnStop, OnSubmit, OnControlSelect>,
    ) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
        OnStop: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        h_stack((
            Self::left(surfaces.attach, actions.on_attach),
            empty().style(|style| style.flex_grow(1.0).min_width(0.0)),
            Self::center(surfaces.vendor, actions.on_control_select),
            empty().style(|style| style.flex_grow(1.0).min_width(0.0)),
            FloemComposerUsageView::render(surfaces.usage),
            Self::right(surfaces.composer, draft, actions.on_stop, actions.on_submit),
        ))
        .style(|style| {
            style
                .width_full()
                .min_width(0.0)
                .gap(CONTROL_ROW_GAP)
                .items_center()
        })
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
            .style(|style| style.gap(CONTROL_ROW_GAP).items_center().flex_shrink(0.0))
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
