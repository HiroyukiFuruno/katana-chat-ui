use super::composer;
use super::types::{
    ChatUiRuntimeEffect, ChatUiRuntimeError, ChatUiRuntimeFrame, ChatUiRuntimeInput,
    ChatUiRuntimeViewport,
};
use crate::{ChatUiHistorySessionSurface, ChatUiSurface};
use katana_ui_core::facade::UiCoreFacade;
use katana_ui_core::raster_host::UiTreeCanvasRenderer;
use katana_ui_core::text_raster::{
    PlatformTextFaceSelection, PlatformTextRasterConfig, PlatformTextRasterizer,
};
use katana_ui_core::text_surface::TextSurface;
use katana_ui_core::theme::{FontToken, ThemeSnapshot};

pub struct ChatUiRuntime {
    pub(super) renderer: UiTreeCanvasRenderer,
    pub(super) canvas_background: u32,
    pub(super) composer_background: u32,
    pub(super) composer_text: [u8; 4],
    pub(super) composer_selection: u32,
    pub(super) rasterizer: PlatformTextRasterizer,
    pub(super) composer: TextSurface,
    pub(super) font: FontToken,
    pub(super) open_dropdown: Option<String>,
    pub(super) permission_open: bool,
    pub(super) permission_request_id: Option<u64>,
    pub(super) tree_scroll_y: u32,
    pub(super) permission_scroll_y: u32,
    pub(super) history_scroll_y: u32,
    history_session_ids: Vec<String>,
}

impl ChatUiRuntime {
    pub fn light(surface: &ChatUiSurface) -> Result<Self, ChatUiRuntimeError> {
        Self::new(ThemeSnapshot::light(), surface)
    }

    pub fn new(theme: ThemeSnapshot, surface: &ChatUiSurface) -> Result<Self, ChatUiRuntimeError> {
        let canvas_background = background_color(&theme);
        let composer_background = packed_color(&theme, "surface", [37, 37, 38, 255]);
        let composer_text = theme_color(&theme, "text", [212, 212, 212, 255]);
        let composer_selection = packed_color(&theme, "selection", [38, 79, 120, 255]);
        let facade = UiCoreFacade::new(theme.clone());
        let font = facade
            .font("body")
            .cloned()
            .ok_or(ChatUiRuntimeError::MissingBodyFont)?;
        let text_raster_config = PlatformTextRasterConfig::default();
        Ok(Self {
            renderer: chat_renderer(theme, text_raster_config.clone()),
            canvas_background,
            composer_background,
            composer_text,
            composer_selection,
            rasterizer: PlatformTextRasterizer::new(text_raster_config),
            composer: composer::text_surface(surface),
            font,
            open_dropdown: None,
            permission_open: false,
            permission_request_id: None,
            tree_scroll_y: 0,
            permission_scroll_y: 0,
            history_scroll_y: 0,
            history_session_ids: Vec::new(),
        })
    }

    pub fn render(
        &mut self,
        surface: &ChatUiSurface,
        viewport: ChatUiRuntimeViewport,
    ) -> Result<ChatUiRuntimeFrame, ChatUiRuntimeError> {
        let _ = self.composer.synchronize_value(&surface.composer.text);
        let request_id = surface
            .vendor_bar
            .permission_request
            .as_ref()
            .map(|request| request.request_id);
        if self.permission_request_id != request_id {
            self.permission_scroll_y = 0;
        }
        self.permission_request_id = request_id;
        self.permission_open = request_id.is_some();
        super::render::frame(self, surface, viewport)
    }

    pub(super) fn sync_history_scroll(
        &mut self,
        visible: bool,
        sessions: &[ChatUiHistorySessionSurface],
        content_height: u32,
        viewport_height: u32,
    ) {
        let session_ids = sessions
            .iter()
            .map(|session| session.session_id.clone())
            .collect::<Vec<_>>();
        if !visible || self.history_session_ids != session_ids {
            self.history_scroll_y = 0;
        }
        self.history_session_ids = session_ids;
        self.history_scroll_y = self
            .history_scroll_y
            .min(content_height.saturating_sub(viewport_height));
    }

    pub fn dispatch(
        &mut self,
        frame: &ChatUiRuntimeFrame,
        input: ChatUiRuntimeInput,
    ) -> Result<Vec<ChatUiRuntimeEffect>, ChatUiRuntimeError> {
        super::input::dispatch(self, frame, input)
    }
}

fn chat_renderer(theme: ThemeSnapshot, config: PlatformTextRasterConfig) -> UiTreeCanvasRenderer {
    UiTreeCanvasRenderer::with_text_raster_config(theme, config, PlatformTextFaceSelection::System)
        .with_code_text_raster_config(
            chat_code_text_raster_config(),
            PlatformTextFaceSelection::CandidateChain,
        )
}

fn chat_code_text_raster_config() -> PlatformTextRasterConfig {
    let mut config = PlatformTextRasterConfig::default();
    config.monospace_candidates = config.proportional_candidates.clone();
    if cfg!(target_os = "macos") {
        // 本文のSystem経路がLatin文字へ選ぶSF NSをcodeにも使い、既存の描画を維持する。
        config.monospace_candidates =
            vec![std::path::PathBuf::from("/System/Library/Fonts/SFNS.ttf")];
    }
    config
}

fn background_color(theme: &ThemeSnapshot) -> u32 {
    packed_color(theme, "background", [30, 30, 30, 255])
}

fn packed_color(theme: &ThemeSnapshot, name: &str, fallback: [u8; 4]) -> u32 {
    let rgba = theme_color(theme, name, fallback);
    (u32::from(rgba[0]) << 16) | (u32::from(rgba[1]) << 8) | u32::from(rgba[2])
}

fn theme_color(theme: &ThemeSnapshot, name: &str, fallback: [u8; 4]) -> [u8; 4] {
    theme.color(name).unwrap_or(fallback)
}

#[cfg(test)]
#[path = "state_contract_tests.rs"]
mod state_contract_tests;

#[cfg(test)]
#[path = "state_geometry_tests.rs"]
mod state_geometry_tests;

#[cfg(test)]
#[path = "state_wrap_tests.rs"]
mod state_wrap_tests;
