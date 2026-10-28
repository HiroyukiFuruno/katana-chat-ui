mod element;
mod image;
mod input;
mod text;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{
    Bounds, ClipboardItem, Context, FocusHandle, Pixels, Render, RenderImage, Window, div,
    prelude::*,
};
use katana_chat_ui::ChatUiSurface;
use katana_chat_ui::kuc::{
    ChatUiRuntime, ChatUiRuntimeClipboard, ChatUiRuntimeEffect, ChatUiRuntimeError,
    ChatUiRuntimeFrame, ChatUiRuntimeInput, ChatUiRuntimeViewport,
};

use self::{element::KucRasterElement, image::render_image};

pub struct GpuiKucView {
    runtime: ChatUiRuntime,
    surface: ChatUiSurface,
    frame: Option<ChatUiRuntimeFrame>,
    image: Option<Arc<RenderImage>>,
    paint_bounds: Option<Bounds<Pixels>>,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) draft: String,
    pub(crate) selection: std::ops::Range<usize>,
    pub(crate) composition: Option<input::CompositionState>,
    ime_trace_seq: u64,
    was_focused: bool,
    on_effect: Box<EffectHandler>,
}

type EffectHandler =
    dyn FnMut(ChatUiRuntimeEffect, &mut Context<GpuiKucView>) -> Option<ChatUiSurface>;

#[derive(Debug)]
pub enum GpuiKucViewError {
    Runtime(ChatUiRuntimeError),
    InvalidRgba,
}

impl std::fmt::Display for GpuiKucViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => error.fmt(formatter),
            Self::InvalidRgba => formatter.write_str("invalid KUC RGBA frame"),
        }
    }
}

impl std::error::Error for GpuiKucViewError {}

impl GpuiKucView {
    pub fn new(
        surface: ChatUiSurface,
        on_effect: impl FnMut(ChatUiRuntimeEffect, &mut Context<Self>) -> Option<ChatUiSurface>
        + 'static,
        cx: &mut Context<Self>,
    ) -> Result<Self, GpuiKucViewError> {
        let runtime = Self::prepare(&surface)?;
        Ok(Self::from_runtime(runtime, surface, on_effect, cx))
    }

    pub fn prepare(surface: &ChatUiSurface) -> Result<ChatUiRuntime, GpuiKucViewError> {
        ChatUiRuntime::light(surface).map_err(GpuiKucViewError::Runtime)
    }

    pub fn from_runtime(
        runtime: ChatUiRuntime,
        surface: ChatUiSurface,
        on_effect: impl FnMut(ChatUiRuntimeEffect, &mut Context<Self>) -> Option<ChatUiSurface>
        + 'static,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            runtime,
            surface,
            frame: None,
            image: None,
            paint_bounds: None,
            focus_handle: cx.focus_handle(),
            draft: String::new(),
            selection: 0..0,
            composition: None,
            ime_trace_seq: 0,
            was_focused: false,
            on_effect: Box::new(on_effect),
        }
    }

    pub fn set_surface(&mut self, surface: ChatUiSurface, cx: &mut Context<Self>) {
        self.surface = surface;
        cx.notify();
    }

    pub(crate) fn redraw(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &Window,
    ) -> Result<(), GpuiKucViewError> {
        let viewport = ChatUiRuntimeViewport::new(
            f32::from(bounds.size.width).max(1.0) as u32,
            f32::from(bounds.size.height).max(1.0) as u32,
            window.scale_factor(),
        );
        let frame = self
            .runtime
            .render(&self.surface, viewport)
            .map_err(GpuiKucViewError::Runtime)?;
        self.image = Some(render_image(&frame.rgba)?);
        let marked = frame.composer.text_surface.preedit.is_some();
        self.frame = Some(frame);
        self.paint_bounds = Some(bounds);
        self.trace_ime_state(marked, "gpui-kuc-runtime-frame");
        self.sync_input_state();
        Ok(())
    }

    fn trace_ime_state(&mut self, marked: bool, source: &str) {
        if std::env::var("KCU_IME_STATE_TRACE").as_deref() != Ok("1") {
            return;
        }
        self.ime_trace_seq = self.ime_trace_seq.saturating_add(1);
        eprintln!(
            "{}",
            ime_state_output(source, self.ime_trace_seq, SystemTime::now(), marked)
        );
    }

    pub(crate) fn local_point(&self, point: gpui::Point<Pixels>) -> Option<gpui::Point<Pixels>> {
        let bounds = self.paint_bounds?;
        Some(gpui::point(point.x - bounds.left(), point.y - bounds.top()))
    }

    pub(crate) fn dispatch_input(
        &mut self,
        input: ChatUiRuntimeInput,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(frame) = self.frame.as_ref() else {
            return;
        };
        match self.runtime.dispatch(frame, input) {
            Ok(effects) => {
                for effect in effects {
                    self.apply_effect(&effect, cx);
                    if let Some(surface) = (self.on_effect)(effect, cx) {
                        self.surface = surface;
                    }
                }
                cx.notify();
                window.refresh();
            }
            Err(error) => eprintln!("KUC input dispatch failed: {error}"),
        }
    }

    fn apply_effect(&mut self, effect: &ChatUiRuntimeEffect, cx: &mut Context<Self>) {
        if let ChatUiRuntimeEffect::DraftChanged(draft) = effect {
            self.draft = draft.clone();
            self.surface.composer.text = draft.clone();
        }
        if let ChatUiRuntimeEffect::Clipboard(ChatUiRuntimeClipboard::Write(text)) = effect {
            cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        }
    }

    fn sync_input_state(&mut self) {
        let Some(frame) = self.frame.as_ref() else {
            return;
        };
        self.draft = self.surface.composer.text.clone();
        self.selection = text::byte_range_to_utf16(
            &self.draft,
            frame.composer.text_surface.selection_start..frame.composer.text_surface.selection_end,
        );
    }
}

impl Render for GpuiKucView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.synchronize_focus(window, cx);
        div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(input::pointer_press))
            .on_mouse_move(cx.listener(input::pointer_drag))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(input::pointer_release))
            .on_key_down(cx.listener(input::key_down))
            .on_scroll_wheel(cx.listener(input::scroll))
            .child(KucRasterElement::new(cx.entity()))
    }
}

impl GpuiKucView {
    fn synchronize_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focused = self.focus_handle.is_focused(window);
        if self.was_focused && !focused {
            self.dispatch_input(ChatUiRuntimeInput::FocusLost, window, cx);
        }
        self.was_focused = focused;
    }
}

fn ime_state_record(
    source: &str,
    sequence: u64,
    observed_at: SystemTime,
    marked: bool,
) -> Result<String, &'static str> {
    let seconds = observed_at
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "before-unix-epoch")?
        .as_secs_f64();
    Ok(format!(
        "KCU_IME_STATE {{\"kind\":\"ime-state\",\"source\":\"{source}\",\"seq\":{sequence},\"observed_at\":{seconds:.6},\"marked\":{marked}}}"
    ))
}

fn ime_state_output(source: &str, sequence: u64, observed_at: SystemTime, marked: bool) -> String {
    match ime_state_record(source, sequence, observed_at, marked) {
        Ok(record) => record,
        Err(error) => format!("KCU_IME_STATE_ERROR source={source} clock={error}"),
    }
}

#[cfg(test)]
mod ime_trace_tests {
    use super::{ime_state_output, ime_state_record};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn formats_typed_state_record() -> Result<(), &'static str> {
        let record = ime_state_record(
            "gpui-native-marked-range",
            7,
            UNIX_EPOCH + Duration::from_secs_f64(12.5),
            true,
        )?;
        assert!(record.contains("\"seq\":7"));
        assert!(record.contains("\"source\":\"gpui-native-marked-range\""));
        assert!(record.ends_with("\"marked\":true}"));
        Ok(())
    }

    #[test]
    fn rejects_pre_epoch_clock_without_state_record() {
        assert_eq!(
            ime_state_record(
                "gpui-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                false,
            ),
            Err("before-unix-epoch")
        );
        assert_eq!(
            ime_state_output(
                "gpui-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                false,
            ),
            "KCU_IME_STATE_ERROR source=gpui-kuc-runtime-frame clock=before-unix-epoch"
        );
    }
}
