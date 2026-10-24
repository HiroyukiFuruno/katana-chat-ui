use crate::{ChatHostError, ChatHostState};
use katana_chat_ui::AgentActivityKind;

pub(super) fn project(
    host: &mut ChatHostState,
    kind: AgentActivityKind,
) -> Result<(), ChatHostError> {
    host.session
        .set_assistant_activity(kind)
        .map_err(ChatHostError::from)
}

#[cfg(test)]
mod tests {
    use super::project;
    use crate::{ChatHistoryStore, ChatHostState, ChatOutputExecutor};
    use katana_chat_ui::{AgentActivityKind, ChatSession, VendorUiState};

    #[test]
    fn typed_activity_reaches_the_streaming_session() -> Result<(), Box<dyn std::error::Error>> {
        for kind in activity_kinds() {
            let mut host = host()?;
            host.session_mut().start_assistant_stream(String::new())?;
            project(&mut host, kind)?;
            assert_eq!(rendered_activity(&host), Some(kind));
        }
        Ok(())
    }

    fn activity_kinds() -> [AgentActivityKind; 7] {
        [
            AgentActivityKind::Processing,
            AgentActivityKind::Generating,
            AgentActivityKind::Editing,
            AgentActivityKind::Reading,
            AgentActivityKind::Searching,
            AgentActivityKind::WebSearching,
            AgentActivityKind::Executing,
        ]
    }

    fn rendered_activity(host: &ChatHostState) -> Option<AgentActivityKind> {
        host.session()
            .render_model()
            .messages
            .last()
            .and_then(|message| message.activity.as_ref().map(|activity| activity.kind))
    }

    fn host() -> Result<ChatHostState, Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(ChatHistoryStore::new_session_id());
        let mut session = ChatSession::new();
        session.set_provider_configured("codex-cli");
        session.set_vendor_ui_state(VendorUiState::for_vendor("codex-cli"));
        Ok(ChatHostState::new(
            session,
            ChatHistoryStore::new(root.clone()),
            ChatOutputExecutor::new(root),
        )?)
    }
}
