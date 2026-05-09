use floem::prelude::*;
use katana_chat_ui::{ChatUiSurface, HostActionKind};

use super::{
    FloemChatActions, FloemChatPanel, FloemComposerActions, FloemComposerView, FloemPanelSlot,
    FloemThreadView, toolbar::FloemToolbar,
};

pub(super) struct FloemChatRoot;

impl FloemChatRoot {
    pub(super) fn render<
        OnAttach,
        OnRemoveAttachment,
        OnNewChat,
        OnHistory,
        OnOutputAction,
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
            OnOutputAction,
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
        OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnStop: Fn() + Copy + 'static,
        OnVendorSelect: Fn(String) + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        container(panel(surface, draft, actions)).style(|style| style.size_full())
    }
}

fn panel<
    OnAttach,
    OnRemoveAttachment,
    OnNewChat,
    OnHistory,
    OnOutputAction,
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
        OnOutputAction,
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
    OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
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
            actions.on_vendor_select,
        )))
        .thread(FloemPanelSlot::new(FloemThreadView::render(
            surface,
            actions.on_output_action,
        )))
        .composer(FloemPanelSlot::new(composer(surface, draft, actions)))
        .render()
}

fn composer<
    OnAttach,
    OnRemoveAttachment,
    OnNewChat,
    OnHistory,
    OnOutputAction,
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
        OnOutputAction,
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
    OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
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
