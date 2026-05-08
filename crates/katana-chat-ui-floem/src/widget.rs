mod action_button;
mod actions;
mod composer;
mod composer_controls;
#[cfg(test)]
mod composer_tests;
mod composer_usage;
mod markdown;
mod output_handoff;
mod panel;
mod provider_icon_selector;
mod root;
mod root_layout;
mod settings;
mod styles;
mod thinking;
mod thinking_indicator;
mod thread;
mod thread_layout;
#[cfg(test)]
mod thread_tests;
mod toolbar;
mod vendor_control_parts;
mod vendor_controls;

pub use action_button::FloemActionIconButton;
pub use actions::FloemChatActions;
pub use composer::{FloemComposerActions, FloemComposerView};
pub use composer_controls::FloemComposerControlsView;
pub use panel::{FloemChatPanel, FloemPanelSlot, FloemPanelSlotView};
pub use thread::FloemThreadView;
pub use vendor_controls::FloemVendorControlsView;

use floem::prelude::*;
use katana_chat_ui::ChatUiSurface;

pub struct FloemChatView;

impl FloemChatView {
    pub fn render<
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
        root::FloemChatRoot::render(surface, draft, actions)
    }
}
