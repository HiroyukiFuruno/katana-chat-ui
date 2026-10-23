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
        self.frame = Some(frame);
        self.image = Some(image);
        self.image_hash = image_hash;
        Ok(())
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
