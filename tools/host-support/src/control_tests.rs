use katana_chat_ui::{ChatSession, ChatUiAction, VendorUiState};

use crate::{ChatHistoryStore, ChatHostState, ChatOutputExecutor};

fn host() -> Result<ChatHostState, Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(ChatHistoryStore::new_session_id());
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("katanagent").with_models(vec!["local".into()], "local"),
    );
    Ok(ChatHostState::new(
        session,
        ChatHistoryStore::new(root.clone()),
        ChatOutputExecutor::new(root),
    )?)
}

#[test]
fn new_chat_preserves_live_connection_and_clears_draft() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = host()?;
    host.session_mut().draft_mut().set_text("old draft");
    host.new_chat();
    assert!(host.session().draft().text.is_empty());
    host.session_mut().draft_mut().set_text("new message");
    assert!(host.session().render_model().input.can_submit);
    host.session_mut().set_provider_missing("disconnected");
    host.new_chat();
    host.session_mut().draft_mut().set_text("must not send");
    assert!(!host.session().render_model().input.can_submit);
    Ok(())
}

#[test]
fn unsupported_selections_preserve_provider_and_model() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = host()?;
    let previous = host.session().vendor_ui_state().clone();
    assert!(
        host.dispatch(ChatUiAction::SelectVendor {
            vendor_id: "unavailable".into()
        })
        .is_err()
    );
    assert!(
        host.dispatch(ChatUiAction::SelectVendorControl {
            key: "model".into(),
            value: "unavailable".into(),
        })
        .is_err()
    );
    assert_eq!(host.session().vendor_ui_state(), &previous);
    Ok(())
}
