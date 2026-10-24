use super::ChatUiSurface;
use crate::{ChatRenderModel, ChatSession};

pub trait ChatUiSurfaceProvider {
    fn chat_ui_surface(&self) -> ChatUiSurface;
}

impl ChatUiSurfaceProvider for ChatRenderModel {
    fn chat_ui_surface(&self) -> ChatUiSurface {
        ChatUiSurface::from_render_model(self)
    }
}

impl ChatUiSurfaceProvider for ChatSession {
    fn chat_ui_surface(&self) -> ChatUiSurface {
        self.render_model().chat_ui_surface()
    }
}
