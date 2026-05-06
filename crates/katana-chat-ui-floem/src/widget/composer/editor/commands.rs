use crate::widget::composer::state::ComposerDraftSubmitter;
use floem::{
    keyboard::Modifiers,
    prelude::*,
    views::editor::command::{Command, CommandExecuted},
};
use floem_editor_core::command::EditCommand;
use katana_chat_ui::ChatUiSurface;

pub(super) struct EditorCommandHandler;

impl EditorCommandHandler {
    pub(super) fn handle_pre_command<OnSubmit>(
        command: &Command,
        modifiers: Modifiers,
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        on_submit: OnSubmit,
    ) -> CommandExecuted
    where
        OnSubmit: Fn(String) + Clone + 'static,
    {
        if !Self::is_submit_command(command, modifiers) {
            return CommandExecuted::No;
        }
        Self::submit_if_allowed(surface, draft, on_submit);
        CommandExecuted::Yes
    }

    pub(super) fn is_submit_command(command: &Command, modifiers: Modifiers) -> bool {
        let is_enter_command = matches!(
            command,
            Command::Edit(EditCommand::InsertNewLine | EditCommand::NewLineBelow)
        );
        is_enter_command && modifiers.meta() && !modifiers.shift()
    }

    fn submit_if_allowed<OnSubmit>(
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        on_submit: OnSubmit,
    ) where
        OnSubmit: Fn(String) + Clone + 'static,
    {
        let composer = surface.get_untracked().composer;
        ComposerDraftSubmitter::submit_allowed(&composer, draft, on_submit);
    }
}
