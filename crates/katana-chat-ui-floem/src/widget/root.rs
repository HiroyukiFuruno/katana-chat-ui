use floem::prelude::*;
use katana_chat_ui::ChatUiSurface;

use super::{
    FloemChatActions, FloemChatPanel, FloemComposerActions, FloemComposerView, FloemPanelSlot,
    FloemThreadView, output_handoff, root_layout, settings::FloemSettingsSurface,
    toolbar::FloemToolbar,
};

pub(super) struct FloemChatRoot;

impl FloemChatRoot {
    pub(super) fn render<
        OnAttach,
        OnRemoveAttachment,
        OnNewChat,
        OnHistory,
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
            OnNewChat,
            OnHistory,
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
        OnNewChat: Fn() + Copy + 'static,
        OnHistory: Fn() + Copy + 'static,
        OnSettings: Fn() + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnStop: Fn() + Copy + 'static,
        OnVendorSelect: Fn(String) + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        let output_hovered = RwSignal::new(false);
        root_with_output_handoff(panel(surface, draft, actions), surface, output_hovered)
    }
}

fn panel<
    OnAttach,
    OnRemoveAttachment,
    OnNewChat,
    OnHistory,
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
        OnNewChat,
        OnHistory,
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
    OnNewChat: Fn() + Copy + 'static,
    OnHistory: Fn() + Copy + 'static,
    OnSettings: Fn() + Copy + 'static,
    OnSubmit: Fn(String) + Clone + 'static,
    OnStop: Fn() + Copy + 'static,
    OnVendorSelect: Fn(String) + Copy + 'static,
    OnControlSelect: Fn(String, String) + Copy + 'static,
{
    FloemChatPanel::new()
        .header(FloemPanelSlot::new(FloemToolbar::render(
            surface,
            actions.on_new_chat,
            actions.on_history,
            actions.on_settings,
            actions.on_vendor_select,
        )))
        .thread(FloemPanelSlot::new(FloemThreadView::render(surface)))
        .composer(FloemPanelSlot::new(composer(surface, draft, actions)))
        .render()
}

fn composer<
    OnAttach,
    OnRemoveAttachment,
    OnNewChat,
    OnHistory,
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
        OnNewChat,
        OnHistory,
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
    OnNewChat: Fn() + Copy + 'static,
    OnHistory: Fn() + Copy + 'static,
    OnSettings: Fn() + Copy + 'static,
    OnSubmit: Fn(String) + Clone + 'static,
    OnStop: Fn() + Copy + 'static,
    OnVendorSelect: Fn(String) + Copy + 'static,
    OnControlSelect: Fn(String, String) + Copy + 'static,
{
    FloemComposerView::composer(
        surface,
        draft,
        FloemComposerActions::new(
            actions.on_attach,
            actions.on_remove_attachment,
            actions.on_submit,
            actions.on_stop,
            actions.on_control_select,
        ),
    )
}

fn root_with_output_handoff(
    panel: impl IntoView + 'static,
    surface: RwSignal<ChatUiSurface>,
    output_hovered: RwSignal<bool>,
) -> impl IntoView {
    container(
        stack((
            root_layout::FloemRootLayout::full_size_layer(panel),
            output_handoff::FloemOutputHandoffHover::render(surface, output_hovered),
            FloemSettingsSurface::render(surface),
        ))
        .style(root_layout::FloemRootLayout::full_size_style),
    )
    .style(|style| style.size_full())
}
