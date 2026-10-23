mod controller;
mod raster;
#[cfg(test)]
mod tests;

pub use controller::EguiChatController;
pub use raster::EguiKucView;
pub use raster::EguiKucView as EguiChatView;
