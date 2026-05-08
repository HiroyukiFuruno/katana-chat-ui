mod commands;

use crate::widget::styles;
use commands::EditorCommandHandler;
use floem::{
    AnyView,
    peniko::Color,
    prelude::*,
    reactive::create_effect,
    views::{
        editor::{Editor, keypress::default_key_handler, text::WrapMethod},
        text_editor::{TextEditor, text_editor_keys},
    },
};
use katana_chat_ui::ChatUiSurface;

#[cfg(test)]
mod tests;

const INPUT_HEIGHT: f64 = 92.0;

pub(super) struct ComposerEditorView;

impl ComposerEditorView {
    pub(super) fn render<OnSubmit>(
        surface: RwSignal<ChatUiSurface>,
        placeholder: String,
        editor_text: String,
        draft: RwSignal<String>,
        on_submit: OnSubmit,
    ) -> impl IntoView
    where
        OnSubmit: Fn(String) + Clone + 'static,
    {
        let reset_revision = reset_revision(draft);
        dyn_container(
            move || reset_revision.get(),
            move |_| {
                editor_instance(
                    surface,
                    placeholder.clone(),
                    editor_text_for_reset(editor_text.clone(), draft),
                    draft,
                    on_submit.clone(),
                )
            },
        )
    }
}

fn editor_instance<OnSubmit>(
    surface: RwSignal<ChatUiSurface>,
    placeholder: String,
    editor_text: String,
    draft: RwSignal<String>,
    on_submit: OnSubmit,
) -> AnyView
where
    OnSubmit: Fn(String) + Clone + 'static,
{
    let editor = editor(editor_text, placeholder, surface, draft, on_submit);
    let editor_for_update = editor.editor().clone();
    editor
        .update(move |event| sync_draft_from_editor(event.editor, &editor_for_update, draft))
        .style(|style| {
            style
                .width_full()
                .height(INPUT_HEIGHT)
                .border(0.0)
                .background(Color::WHITE)
                .font_size(styles::FONT_BODY)
        })
        .into_any()
}

fn reset_revision(draft: RwSignal<String>) -> RwSignal<u64> {
    let revision = RwSignal::new(0_u64);
    let had_content = RwSignal::new(!draft.get_untracked().is_empty());
    create_effect(move |_| update_reset_revision(draft, had_content, revision));
    revision
}

fn update_reset_revision(
    draft: RwSignal<String>,
    had_content: RwSignal<bool>,
    revision: RwSignal<u64>,
) {
    if !draft.get().is_empty() {
        had_content.set(true);
        return;
    }
    if had_content.get_untracked() {
        had_content.set(false);
        revision.update(|it| *it = it.saturating_add(1));
    }
}

fn editor_text_for_reset(initial_text: String, draft: RwSignal<String>) -> String {
    let current_draft = draft.get_untracked();
    if current_draft.is_empty() {
        return String::new();
    }
    if current_draft == initial_text {
        return initial_text;
    }
    current_draft
}

fn editor<OnSubmit>(
    editor_text: String,
    placeholder: String,
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    on_submit: OnSubmit,
) -> TextEditor
where
    OnSubmit: Fn(String) + Clone + 'static,
{
    text_editor_keys(editor_text, move |editor_signal, keypress, modifiers| {
        if EditorCommandHandler::is_submit_keypress(keypress, modifiers) {
            sync_draft_from_editor_signal(editor_signal, draft);
            return EditorCommandHandler::handle_submit_keypress(
                keypress,
                modifiers,
                surface,
                draft,
                on_submit.clone(),
            );
        }
        default_key_handler(editor_signal)(keypress, modifiers)
    })
    .placeholder(placeholder)
    .editor_style(|style| {
        style
            .hide_gutter(true)
            .wrap_method(WrapMethod::EditorWidth)
            .placeholder_color(styles::COLOR_MUTED)
            .cursor_color(styles::COLOR_TEXT)
            .preedit_underline_color(styles::COLOR_TEXT)
    })
}

fn sync_draft_from_editor(
    editor: Option<&floem::views::editor::Editor>,
    fallback_editor: &floem::views::editor::Editor,
    draft: RwSignal<String>,
) {
    let editor = editor.unwrap_or(fallback_editor);
    draft.set(editor.rope_text().text.to_string());
}

fn sync_draft_from_editor_signal(editor_signal: RwSignal<Editor>, draft: RwSignal<String>) {
    editor_signal.with_untracked(|editor| {
        draft.set(editor.rope_text().text.to_string());
    });
}
