//! Floem renderer for the katana-chat-ui standard chat UI.
//!
//! Host applications mount `FloemChatView::mount`. KUC owns the conversation UI;
//! the Floem host supplies OS input, clipboard, and the raster image surface.

mod raster_view;

pub use raster_view::{FloemChatView, FloemKucCallbacks};
