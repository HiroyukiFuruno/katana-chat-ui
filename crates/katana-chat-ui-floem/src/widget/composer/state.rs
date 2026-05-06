use floem::prelude::*;
use katana_chat_ui::ChatUiComposerSurface;

pub(crate) struct ComposerDraftSubmitter;

impl ComposerDraftSubmitter {
    pub(crate) fn submit<OnSubmit>(draft: RwSignal<String>, on_submit: OnSubmit)
    where
        OnSubmit: Fn(String) + Clone + 'static,
    {
        let text = draft.get();
        if !text.trim().is_empty() {
            on_submit(text);
        }
    }

    pub(crate) fn submit_allowed<OnSubmit>(
        composer: &ChatUiComposerSurface,
        draft: RwSignal<String>,
        on_submit: OnSubmit,
    ) where
        OnSubmit: Fn(String) + Clone + 'static,
    {
        let text = draft.get();
        if !ComposerSubmitGate::can_submit(composer, &text) {
            return;
        }
        if text.trim().is_empty() {
            on_submit(text);
            return;
        }
        Self::submit(draft, on_submit);
    }
}

pub(crate) struct ComposerEditorState;

impl ComposerEditorState {
    pub(crate) fn initial_text(
        composer: &ChatUiComposerSurface,
        draft: RwSignal<String>,
    ) -> String {
        let live_draft = draft.get_untracked();
        if live_draft.is_empty() {
            return composer.text.clone();
        }
        live_draft
    }
}

pub(crate) struct ComposerSubmitGate;

impl ComposerSubmitGate {
    pub(crate) fn can_submit(composer: &ChatUiComposerSurface, draft: &str) -> bool {
        composer.send_available
            && !composer.stop_enabled
            && (!draft.trim().is_empty() || !composer.attachments.is_empty())
    }
}
