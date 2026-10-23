use super::super::ChatUiAction;
use katana_ui_core::raster_host::{UiTreeHostActionHit, UiTreeNodeHit};
use katana_ui_core::render_model::{UiRect, UiTree};
use katana_ui_core::text_surface::{TextSurfaceFrameRecord, TextSurfaceLayout};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatUiRuntimeViewport {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

impl ChatUiRuntimeViewport {
    #[must_use]
    pub const fn new(width: u32, height: u32, scale_factor: f32) -> Self {
        Self {
            width,
            height,
            scale_factor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatUiRuntimeRgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatUiRuntimeComposerFrame {
    pub bounds: UiRect,
    pub layout: TextSurfaceLayout,
    pub text_surface: TextSurfaceFrameRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatUiRuntimeFrame {
    pub rgba: ChatUiRuntimeRgba,
    pub tree: UiTree,
    pub viewport: ChatUiRuntimeViewport,
    pub composer: ChatUiRuntimeComposerFrame,
    pub action_hits: Vec<UiTreeHostActionHit>,
    pub node_hits: Vec<UiTreeNodeHit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatUiRuntimeKey {
    Enter { shift: bool, command: bool },
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    CommandA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatUiRuntimeCompositionPhase {
    Start,
    Update,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatUiRuntimeInput {
    PointerPress {
        x: f32,
        y: f32,
        extend_selection: bool,
    },
    PointerDrag {
        x: f32,
        y: f32,
    },
    PointerRelease,
    Text(String),
    Key(ChatUiRuntimeKey),
    ImeComposition {
        phase: ChatUiRuntimeCompositionPhase,
        preedit: String,
        /// KUCへ渡すため、preedit内のUTF-8バイト位置を指定する。
        /// OSの文字indexやUTF-16位置はadapterで変換する。
        caret: usize,
    },
    ImeCommit(String),
    ImeCommitRange {
        start_byte: usize,
        end_byte: usize,
        text: String,
    },
    ImeCancel,
    FocusLost,
    ClipboardPaste(String),
    ClipboardCopy,
    ClipboardCut,
    Scroll {
        x: f32,
        y: f32,
        delta_x: i32,
        delta_y: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatUiRuntimeClipboard {
    Read,
    Write(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatUiRuntimeEffect {
    Action(ChatUiAction),
    DraftChanged(String),
    Clipboard(ChatUiRuntimeClipboard),
    FocusComposer,
    Repaint,
}

#[derive(Debug, Error)]
pub enum ChatUiRuntimeError {
    #[error("KUC theme does not provide the body font")]
    MissingBodyFont,
    #[error("KUC tree did not produce the composer input bounds")]
    MissingComposerBounds,
    #[error("KUC tree did not produce {component} for message {message_id}")]
    MissingMessageGeometry {
        message_id: String,
        component: String,
    },
    #[error("KUC renderer did not produce history panel content bounds")]
    MissingHistoryGeometry,
    #[error("KUC text raster failed: {0}")]
    TextRaster(#[from] katana_ui_core::text_raster::PlatformTextRasterError),
    #[error("KUC chat action failed: {0}")]
    Action(#[from] super::super::ChatUiActionError),
}
