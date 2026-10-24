use super::dispatch;
use crate::{
    ChatHistoryStore, ChatHostEffect, ChatHostState, ChatOutputExecutor, ManagedHostCommand,
    ProviderManager,
};
use katana_chat_ui::{
    Attachment, ChatOutputKind, ChatSession, ChatUiAction, FileCandidateOutput, FileResource,
    MessageStatus, VendorUiState,
};

#[test]
fn managed_dispatch_persists_and_removes_programmatic_markdown_attachment()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let attachment = markdown_attachment();
    let mut host = load_host(root.path())?;
    let mut providers = ProviderManager::new();
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;

    runtime.block_on(dispatch(
        ManagedHostCommand::AddAttachment(attachment.clone()),
        &mut host,
        &mut providers,
    ))?;
    assert_eq!(host.session().draft().attachments, vec![attachment.clone()]);
    assert_eq!(
        load_host(root.path())?.session().draft().attachments,
        vec![attachment]
    );

    runtime.block_on(dispatch(
        ManagedHostCommand::Action(ChatUiAction::RemoveAttachment { index: 0 }),
        &mut host,
        &mut providers,
    ))?;
    assert!(
        load_host(root.path())?
            .session()
            .draft()
            .attachments
            .is_empty()
    );
    Ok(())
}

#[test]
fn managed_dispatch_persists_draft_changed_for_restored_host()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let mut host = load_host(root.path())?;
    let mut providers = ProviderManager::new();
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;

    runtime.block_on(dispatch(
        ManagedHostCommand::DraftChanged("saved draft".into()),
        &mut host,
        &mut providers,
    ))?;

    assert_eq!(
        load_host(root.path())?.session().draft().text,
        "saved draft"
    );
    Ok(())
}

#[test]
fn draft_changed_persistence_failure_keeps_live_draft() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let blocked = root.path().join("history-file");
    std::fs::write(&blocked, "not a directory")?;
    let mut host = load_host(root.path())?;
    host.history = ChatHistoryStore::new(blocked);
    let mut providers = ProviderManager::new();
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;

    let result = runtime.block_on(dispatch(
        ManagedHostCommand::DraftChanged("live draft".into()),
        &mut host,
        &mut providers,
    ));

    assert!(matches!(result, Err(crate::ChatHostError::History(_))));
    assert_eq!(host.session().draft().text, "live draft");
    Ok(())
}

#[test]
fn binding_restored_history_preserves_model_without_provider_connection()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let (session_id, saved) = save_history(root.path())?;
    let mut host = load_host(root.path())?;
    host.dispatch(ChatUiAction::NewChat)?;
    assert!(host.session().snapshot().messages.is_empty());

    let mut providers = ProviderManager::new();
    let effect = host.dispatch(ChatUiAction::SelectHistory { session_id })?;
    assert_eq!(effect, ChatHostEffect::None);
    assert_eq!(
        host.session().snapshot().vendor_state.selected_model,
        saved.vendor_state.selected_model
    );
    providers.bind_session_identity(&mut host)?;
    let restored = host.session().snapshot();
    assert_eq!(restored.messages, saved.messages);
    assert_eq!(restored.outputs, saved.outputs);
    assert_eq!(
        restored.vendor_state.selected_model,
        saved.vendor_state.selected_model
    );
    assert!(restored.messages.iter().all(|message| {
        !matches!(
            message.status,
            MessageStatus::Sending | MessageStatus::Streaming
        )
    }));
    assert_eq!(host.surface().messages.len(), saved.messages.len());
    Ok(())
}

fn markdown_attachment() -> Attachment {
    Attachment::file(FileResource::new(
        "file:///workspace/tmp/source.md",
        "text/markdown",
        "# source",
        8,
    ))
}

fn load_host(root: &std::path::Path) -> Result<ChatHostState, Box<dyn std::error::Error>> {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session.set_vendor_ui_state(VendorUiState::for_vendor("katanagent"));
    Ok(ChatHostState::new(
        session,
        ChatHistoryStore::new(root.to_path_buf()),
        ChatOutputExecutor::new(root.to_path_buf()),
    )?)
}

fn save_history(
    root: &std::path::Path,
) -> Result<(String, katana_chat_ui::ChatSessionSnapshot), Box<dyn std::error::Error>> {
    let mut host = load_host(root)?;
    host.session_mut().set_vendor_ui_state(
        VendorUiState::for_vendor("katanagent")
            .with_models(vec!["ollama:test-model".into()], "ollama:test-model"),
    );
    host.session_mut()
        .draft_mut()
        .set_text("saved user request");
    host.session_mut().submit_draft()?;
    let assistant_id = host
        .session_mut()
        .start_assistant_stream("saved assistant response")?;
    host.session_mut().add_output(
        assistant_id,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/saved.md",
            "text/markdown",
            "# saved",
        )),
    )?;
    host.session_mut().finish_assistant_message()?;
    host.persist()?;

    let record = ChatHistoryStore::new(root.to_path_buf())
        .load_latest()?
        .ok_or("saved history is missing")?;
    Ok((record.session_id, record.snapshot))
}
