//! Floem の OS 入力を KUC raster runtime へ接続する native host view。

use floem::{View, reactive::RwSignal, views::Decorators};
use katana_chat_ui::{ChatUiAction, ChatUiSurface, kuc::ChatUiRuntimeError};

mod events;
mod host;
mod image;

#[cfg(test)]
mod tests;

/// KUC runtime が返す状態変化を host application へ渡す callback 群。
pub struct FloemKucCallbacks<OnAction, OnDraft> {
    pub on_action: OnAction,
    pub on_draft: OnDraft,
}

/// Floem host の標準 KUC mount 入口。
pub struct FloemChatView;

impl FloemChatView {
    pub fn mount<OnAction, OnDraft>(
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        scale_factor: f32,
        callbacks: FloemKucCallbacks<OnAction, OnDraft>,
    ) -> Result<impl View, ChatUiRuntimeError>
    where
        OnAction: Fn(ChatUiAction) + 'static,
        OnDraft: Fn(String) + 'static,
    {
        host::RasterChatView::new(surface, draft, scale_factor, callbacks)
            .map(|view| view.style(raster_view_style))
    }
}

fn raster_view_style(style: floem::style::Style) -> floem::style::Style {
    style.size_full().min_width(0.0).min_height(0.0)
}
