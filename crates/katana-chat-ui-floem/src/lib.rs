//! Floem renderer for the katana-chat-ui standard chat UI.
//!
//! Host applications mount `FloemChatView`. They do not rebuild the conversation
//! UI from descriptors.

mod composer;
mod message_list;
mod output_list;
mod panel;
mod usage;
mod widget;

pub use composer::ComposerView;
pub use message_list::MessageListView;
pub use output_list::{OutputListView, OutputRowView};
pub use panel::ChatPanelView;
pub use usage::UsageMeterView;
pub use widget::FloemChatActions;
pub use widget::FloemChatView;
pub use widget::{
    FloemActionIconButton, FloemChatPanel, FloemComposerControlsView, FloemComposerView,
    FloemPanelSlot, FloemPanelSlotView, FloemThreadView, FloemVendorControlsView,
};
