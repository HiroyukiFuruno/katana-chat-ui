mod composer;
mod effects;
mod key;
mod pointer;
mod scroll;

use super::state::ChatUiRuntime;
use super::types::{
    ChatUiRuntimeEffect, ChatUiRuntimeError, ChatUiRuntimeFrame, ChatUiRuntimeInput,
    ChatUiRuntimeKey,
};

pub(super) fn dispatch(
    runtime: &mut ChatUiRuntime,
    frame: &ChatUiRuntimeFrame,
    input: ChatUiRuntimeInput,
) -> Result<Vec<ChatUiRuntimeEffect>, ChatUiRuntimeError> {
    if !frame.composer.submit_enabled
        && matches!(
            input,
            ChatUiRuntimeInput::Key(ChatUiRuntimeKey::Enter { command: true, .. })
        )
    {
        return Ok(Vec::new());
    }
    if runtime.permission_open && blocks_input_behind_permission(&input) {
        return Ok(Vec::new());
    }
    let mut effects = Vec::new();
    let input = pointer::dispatch(runtime, frame, input, &mut effects)?;
    let Some(input) = input else {
        return Ok(effects);
    };
    let input = scroll::dispatch(runtime, frame, input, &mut effects);
    if let Some(input) = input {
        composer::dispatch(runtime, input, &mut effects);
    }
    Ok(effects)
}

fn blocks_input_behind_permission(input: &ChatUiRuntimeInput) -> bool {
    !matches!(
        input,
        ChatUiRuntimeInput::PointerPress { .. }
            | ChatUiRuntimeInput::Scroll { .. }
            | ChatUiRuntimeInput::ImeCancel
            | ChatUiRuntimeInput::FocusLost
    )
}
