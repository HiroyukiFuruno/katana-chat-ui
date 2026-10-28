use floem::{
    View, ViewId,
    context::{EventCx, PaintCx},
    event::{Event, EventPropagation},
    reactive::{RwSignal, SignalGet, SignalUpdate},
};
use floem_renderer::{Img, Renderer};
use katana_chat_ui::{
    ChatUiAction, ChatUiSurface,
    kuc::{
        ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeError, ChatUiRuntimeFrame,
        ChatUiRuntimeInput, ChatUiRuntimeViewport,
    },
};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    FloemKucCallbacks,
    events::{event_input, is_paste},
    image::image,
};

mod platform;

pub(super) struct RasterChatView<OnAction, OnDraft> {
    id: ViewId,
    runtime: ChatUiRuntime,
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    scale_factor: f32,
    callbacks: FloemKucCallbacks<OnAction, OnDraft>,
    frame: Option<ChatUiRuntimeFrame>,
    image: Option<floem::peniko::Image>,
    image_hash: Vec<u8>,
    ime_trace_seq: u64,
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

impl<OnAction, OnDraft> RasterChatView<OnAction, OnDraft>
where
    OnAction: Fn(ChatUiAction) + 'static,
    OnDraft: Fn(String) + 'static,
{
    pub(super) fn new(
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        scale_factor: f32,
        callbacks: FloemKucCallbacks<OnAction, OnDraft>,
    ) -> Result<Self, ChatUiRuntimeError> {
        let initial = surface.get_untracked();
        let id = ViewId::new();
        id.keyboard_navigable();
        Ok(Self {
            id,
            runtime: ChatUiRuntime::light(&initial)?,
            surface,
            draft,
            scale_factor: if scale_factor > 0.0 {
                scale_factor
            } else {
                1.0
            },
            callbacks,
            frame: None,
            image: None,
            image_hash: Vec::new(),
            ime_trace_seq: 0,
        })
    }

    fn refresh(&mut self) -> Result<bool, ChatUiRuntimeError> {
        let Some(viewport) = self.viewport() else {
            return Ok(false);
        };
        self.render(viewport)?;
        Ok(true)
    }

    fn render(&mut self, viewport: ChatUiRuntimeViewport) -> Result<(), ChatUiRuntimeError> {
        let mut surface = self.surface.get_untracked();
        surface
            .composer
            .text
            .clone_from(&self.draft.get_untracked());
        let frame = self.runtime.render(&surface, viewport)?;
        let (image, image_hash) = image(&frame.rgba);
        self.trace_ime_state(frame.composer.text_surface.preedit.is_some());
        self.frame = Some(frame);
        self.image = Some(image);
        self.image_hash = image_hash;
        Ok(())
    }

    fn trace_ime_state(&mut self, marked: bool) {
        if env::var("KCU_IME_STATE_TRACE").as_deref() != Ok("1") {
            return;
        }
        let source = "floem-kuc-runtime-frame";
        self.ime_trace_seq = self.ime_trace_seq.saturating_add(1);
        eprintln!(
            "{}",
            ime_state_output(source, self.ime_trace_seq, SystemTime::now(), marked)
        );
    }

    fn viewport(&self) -> Option<ChatUiRuntimeViewport> {
        let rect = self.id.get_content_rect();
        (rect.width() > 0.0 && rect.height() > 0.0).then(|| {
            ChatUiRuntimeViewport::new(
                rect.width().ceil() as u32,
                rect.height().ceil() as u32,
                self.scale_factor,
            )
        })
    }

    fn report_runtime_error(&self, error: ChatUiRuntimeError) {
        eprintln!("katana-chat-ui-floem: KUC runtime failed: {error}");
    }

    fn dispatch(&mut self, input: ChatUiRuntimeInput) {
        if self.frame.is_none()
            && let Err(error) = self.refresh()
        {
            self.report_runtime_error(error);
            return;
        }
        if let Some(frame) = self.frame.clone() {
            match self.runtime.dispatch(&frame, input) {
                Ok(effects) => self.apply_effects(effects),
                Err(error) => self.report_runtime_error(error),
            }
        }
    }

    fn apply_effects(&mut self, effects: Vec<ChatUiRuntimeEffect>) {
        for effect in effects {
            self.apply_effect(effect);
        }
    }

    fn apply_effect(&mut self, effect: ChatUiRuntimeEffect) {
        match effect {
            ChatUiRuntimeEffect::Action(action) => (self.callbacks.on_action)(action),
            ChatUiRuntimeEffect::DraftChanged(text) => self.update_draft(text),
            ChatUiRuntimeEffect::Clipboard(effect) => self.apply_clipboard(effect),
            ChatUiRuntimeEffect::FocusComposer => self.id.request_focus(),
            ChatUiRuntimeEffect::Repaint => self.id.request_paint(),
        }
    }

    fn update_draft(&self, text: String) {
        self.draft.set(text.clone());
        (self.callbacks.on_draft)(text);
    }
}

impl<OnAction, OnDraft> View for RasterChatView<OnAction, OnDraft>
where
    OnAction: Fn(ChatUiAction) + 'static,
    OnDraft: Fn(String) + 'static,
{
    fn id(&self) -> ViewId {
        self.id
    }

    fn event_before_children(&mut self, cx: &mut EventCx, event: &Event) -> EventPropagation {
        if matches!(event, Event::FocusLost) {
            floem::action::set_ime_allowed(false);
        }
        if let Some(input) = event_input(event, cx.is_active(self.id)) {
            if matches!(input, ChatUiRuntimeInput::PointerPress { .. }) {
                self.id.request_active();
            }
            if matches!(input, ChatUiRuntimeInput::PointerRelease) {
                self.id.clear_active();
            }
            self.dispatch(input);
            return EventPropagation::Stop;
        }
        if is_paste(event) {
            self.paste();
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        match self.refresh() {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                self.report_runtime_error(error);
                return;
            }
        }
        if cx.is_focused(self.id) {
            self.set_ime_cursor();
        }
        if let Some(image) = &self.image {
            cx.draw_img(
                Img {
                    img: image.clone(),
                    hash: &self.image_hash,
                },
                self.id.get_content_rect(),
            );
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod ime_trace_tests {
    use super::{ime_state_output, ime_state_record};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn formats_typed_state_record() -> Result<(), &'static str> {
        let record = ime_state_record(
            "floem-kuc-runtime-frame",
            7,
            UNIX_EPOCH + Duration::from_secs_f64(12.5),
            false,
        )?;
        assert!(record.contains("\"source\":\"floem-kuc-runtime-frame\""));
        assert!(record.contains("\"observed_at\":12.500000"));
        assert!(record.ends_with("\"marked\":false}"));
        Ok(())
    }

    #[test]
    fn rejects_pre_epoch_clock_without_state_record() {
        assert_eq!(
            ime_state_record(
                "floem-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                true,
            ),
            Err("before-unix-epoch")
        );
        assert_eq!(
            ime_state_output(
                "floem-kuc-runtime-frame",
                8,
                UNIX_EPOCH - Duration::from_secs(1),
                true,
            ),
            "KCU_IME_STATE_ERROR source=floem-kuc-runtime-frame clock=before-unix-epoch"
        );
    }
}
