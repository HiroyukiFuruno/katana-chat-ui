mod commands;

use crate::widget::styles;
use commands::EditorCommandHandler;
use floem::{
    AnyView,
    peniko::Color,
    prelude::*,
    reactive::create_effect,
    views::{
        editor::text::WrapMethod,
        text_editor::{TextEditor, text_editor},
    },
};
use katana_chat_ui::ChatUiSurface;

#[cfg(test)]
mod tests;

const INPUT_HEIGHT: f64 = 92.0;
const PLACEHOLDER_TOP: f64 = 0.0;
const PLACEHOLDER_LEFT: f64 = 0.0;
const PLACEHOLDER_LAYER: i32 = 1;
const EDITOR_LAYER: i32 = 2;

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
    let command_submit = on_submit;
    let editor = editor(editor_text);
    let editor_view = editor
        .pre_command(move |event| {
            EditorCommandHandler::handle_pre_command(
                event.cmd,
                event.mods,
                surface,
                draft,
                command_submit.clone(),
            )
        })
        .update(move |event| sync_draft_from_editor(event.editor, draft))
        .style(|style| {
            style
                .width_full()
                .height(INPUT_HEIGHT)
                .border(0.0)
                .background(Color::WHITE)
                .font_size(styles::FONT_BODY)
        });
    let editor_layer = editor_view.style(|style| style.z_index(EDITOR_LAYER));
    editor_frame(placeholder_view(placeholder, draft), editor_layer).into_any()
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

fn editor_frame(
    placeholder_layer: AnyView,
    editor_layer: impl IntoView + 'static,
) -> impl IntoView {
    stack((placeholder_layer, editor_layer)).style(|style| {
        style
            .width_full()
            .height(INPUT_HEIGHT)
            .position(floem::style::Position::Relative)
    })
}

fn placeholder_view(placeholder: String, draft: RwSignal<String>) -> AnyView {
    dyn_container(
        move || draft.get().is_empty(),
        move |is_empty| {
            if !is_empty {
                return empty().into_any();
            }
            text(placeholder.clone())
                .style(|style| {
                    style
                        .font_size(styles::FONT_BODY)
                        .color(styles::COLOR_MUTED)
                })
                .into_any()
        },
    )
    .style(|style| {
        style
            .absolute()
            .inset_top(PLACEHOLDER_TOP)
            .inset_left(PLACEHOLDER_LEFT)
            .z_index(PLACEHOLDER_LAYER)
    })
    .pointer_events(|| false)
    .into_any()
}

fn editor(editor_text: String) -> TextEditor {
    text_editor(editor_text).editor_style(|style| {
        style
            .hide_gutter(true)
            .wrap_method(WrapMethod::EditorWidth)
            .placeholder_color(styles::COLOR_MUTED)
            .cursor_color(styles::COLOR_TEXT)
            .preedit_underline_color(styles::COLOR_TEXT)
    })
}

fn sync_draft_from_editor(editor: Option<&floem::views::editor::Editor>, draft: RwSignal<String>) {
    if let Some(editor) = editor {
        draft.set(editor.rope_text().text.to_string());
    }
}
