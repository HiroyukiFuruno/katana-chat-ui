use crate::widget::composer::state::ComposerDraftSubmitter;
use floem::{
    keyboard::{Key, KeyCode, Modifiers, NamedKey, PhysicalKey},
    prelude::*,
    views::editor::{
        command::CommandExecuted,
        keypress::{key::KeyInput, press::KeyPress},
    },
};
use katana_chat_ui::ChatUiSurface;

pub(super) struct EditorCommandHandler;

impl EditorCommandHandler {
    pub(super) fn is_submit_keypress(keypress: &KeyPress, modifiers: Modifiers) -> bool {
        (modifiers.meta() || keypress.mods.meta())
            && !(modifiers.shift() || keypress.mods.shift())
            && Self::is_enter_keypress(keypress)
    }

    pub(super) fn handle_submit_keypress<OnSubmit>(
        keypress: &KeyPress,
        modifiers: Modifiers,
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        on_submit: OnSubmit,
    ) -> CommandExecuted
    where
        OnSubmit: Fn(String) + Clone + 'static,
    {
        if !Self::is_submit_keypress(keypress, modifiers) {
            return CommandExecuted::No;
        }
        Self::submit_if_allowed(surface, draft, on_submit);
        CommandExecuted::Yes
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

    fn is_enter_keypress(keypress: &KeyPress) -> bool {
        matches!(
            &keypress.key,
            KeyInput::Keyboard(Key::Named(NamedKey::Enter), _)
                | KeyInput::Keyboard(_, PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter))
        )
    }
}
