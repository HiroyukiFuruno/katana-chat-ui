use std::{cell::RefCell, rc::Rc};

use floem::{
    keyboard::{Key, NamedKey},
    reactive::RwSignal,
};
use katana_chat_ui::{
    ChatUiAction,
    kuc::{ChatUiRuntimeInput, ChatUiRuntimeKey},
};

use super::super::super::events::key_input_parts;
use super::super::RasterChatView;
use super::{
    fixture::{callbacks, ready_surface, viewport},
    focus,
};

#[test]
fn command_enter_repeat_invokes_the_floem_submit_callback_once() -> Result<(), String> {
    let actions = Rc::new(RefCell::new(Vec::new()));
    let drafts = Rc::new(RefCell::new(Vec::new()));
    let mut view = test_view(actions.clone(), drafts)?;
    view.render(viewport()).map_err(|error| error.to_string())?;
    focus(&mut view)?;

    dispatch_command_enter(&mut view, false);
    dispatch_command_enter(&mut view, true);

    assert_submit_actions(&actions, 1)
}

#[test]
fn repeated_plain_enter_remains_a_newline_input() {
    assert_eq!(
        key_input_parts(&Key::Named(NamedKey::Enter), None, false, false, true),
        Some(ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Enter {
            shift: false,
            command: false,
        }))
    );
}

fn test_view(
    actions: Rc<RefCell<Vec<ChatUiAction>>>,
    drafts: Rc<RefCell<Vec<String>>>,
) -> Result<RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>, String> {
    RasterChatView::new(
        RwSignal::new(ready_surface()),
        RwSignal::new("submit once".to_string()),
        1.0,
        callbacks(actions, drafts),
    )
    .map_err(|error| error.to_string())
}

fn dispatch_command_enter(
    view: &mut RasterChatView<impl Fn(ChatUiAction) + 'static, impl Fn(String) + 'static>,
    repeat: bool,
) {
    let input = key_input_parts(&Key::Named(NamedKey::Enter), None, false, true, repeat);
    if let Some(input) = input {
        view.dispatch(input);
    }
}

fn assert_submit_actions(
    actions: &Rc<RefCell<Vec<ChatUiAction>>>,
    count: usize,
) -> Result<(), String> {
    let actual = actions
        .borrow()
        .iter()
        .filter(|action| matches!(action, ChatUiAction::Submit))
        .count();
    if actual == count {
        return Ok(());
    }
    Err(format!("unexpected Floem submit callback count: {actual}"))
}
