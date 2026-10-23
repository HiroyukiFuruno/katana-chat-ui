use std::{
    error::Error,
    path::Path,
    time::{Duration, Instant},
};

use katana_chat_ui::{ChatSession, ChatUiAction, ChatUiSurface, MessageRole};
use kcu_host_support::{
    ChatHistoryStore, ChatHostState, ChatOutputExecutor, ManagedChatHost, ManagedHostCommand,
    ProviderManager,
};
use tokio::{runtime::Builder, time};

const CODEX: &str = "codex-cli";
const DEADLINE: Duration = Duration::from_secs(90);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

fn main() -> Result<(), Box<dyn Error>> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    require_empty_trace()?;
    require_clean_trace()?;
    let root = tempfile::tempdir()?;
    let token = continuity_token();
    first_remote_turn(root.path(), &token).await?;
    managed_history_selection(root.path()).await?;
    restored_remote_turns(root.path(), &token).await?;
    require_clean_trace()?;
    require_trace_evidence()?;
    println!("ACP history restore probe passed");
    Ok(())
}

async fn managed_history_selection(root: &Path) -> Result<(), Box<dyn Error>> {
    let history = ChatHistoryStore::new(root.into());
    let record = history
        .list()?
        .into_iter()
        .next()
        .ok_or("saved ACP history is unavailable")?;
    let session_id = record.session_id;
    let raw_snapshot = record.snapshot;
    exercise_boundaries(root, &session_id, &raw_snapshot).await?;
    let restored = history
        .load(&session_id)?
        .ok_or("selected history disappeared")?;
    let expected_raw = snapshot_with_draft(raw_snapshot, "draft before Managed NewChat");
    if !same_saved_snapshot(&expected_raw, &restored.snapshot) {
        return Err("Managed SelectHistory changed saved snapshot fields".into());
    }
    require_clean_trace()?;
    Ok(())
}

async fn exercise_boundaries(
    root: &Path,
    session_id: &str,
    raw_snapshot: &katana_chat_ui::ChatSessionSnapshot,
) -> Result<(), Box<dyn Error>> {
    let expected = expected_surface(raw_snapshot);
    let host = ManagedChatHost::spawn(root)?;
    let seq = host.send(ManagedHostCommand::Action(ChatUiAction::SelectHistory {
        session_id: session_id.to_string(),
    }))?;
    let update = wait_for_ack(&host, seq).await?;
    require_restored_messages(&expected, &update)?;
    require_model(&raw_snapshot.vendor_state.selected_model, &update)?;
    if !codex_ready(&update) {
        wait_codex_ready(&host).await?;
    }
    prepare_new_chat_draft(&host).await?;
    let new_chat = host.send(ManagedHostCommand::Action(ChatUiAction::NewChat))?;
    let update = wait_for_ack(&host, new_chat).await?;
    require_model(&raw_snapshot.vendor_state.selected_model, &update)?;
    if !update.surface.message_list.messages.is_empty() || !update.surface.composer.text.is_empty()
    {
        return Err("Managed NewChat did not separate messages and draft".into());
    }
    restore_selected_history(&host, session_id.to_string(), &expected, raw_snapshot).await?;
    Ok(())
}

async fn prepare_new_chat_draft(host: &ManagedChatHost) -> Result<(), Box<dyn Error>> {
    let seq = host.send(ManagedHostCommand::DraftChanged(
        "draft before Managed NewChat".into(),
    ))?;
    let update = wait_for_ack(host, seq).await?;
    if update.surface.composer.text == "draft before Managed NewChat" {
        Ok(())
    } else {
        Err("Managed draft update was not acknowledged".into())
    }
}

async fn restore_selected_history(
    host: &ManagedChatHost,
    session_id: String,
    expected: &ChatUiSurface,
    raw: &katana_chat_ui::ChatSessionSnapshot,
) -> Result<(), Box<dyn Error>> {
    let seq = host.send(ManagedHostCommand::Action(ChatUiAction::SelectHistory {
        session_id,
    }))?;
    let update = wait_for_ack(host, seq).await?;
    require_restored_messages(expected, &update)?;
    require_model(&raw.vendor_state.selected_model, &update)
}

fn snapshot_with_draft(
    mut snapshot: katana_chat_ui::ChatSessionSnapshot,
    draft: &str,
) -> katana_chat_ui::ChatSessionSnapshot {
    snapshot.draft.set_text(draft);
    snapshot
}

fn require_model(
    expected: &Option<String>,
    update: &kcu_host_support::ManagedHostUpdate,
) -> Result<(), Box<dyn Error>> {
    let actual = update
        .surface
        .vendor_controls
        .model
        .as_ref()
        .and_then(|model| model.selected_value.as_ref());
    if actual == expected.as_ref() {
        Ok(())
    } else {
        Err(format!(
            "Managed boundary changed selected model: expected={expected:?}, actual={actual:?}"
        )
        .into())
    }
}

fn require_restored_messages(
    expected: &ChatUiSurface,
    update: &kcu_host_support::ManagedHostUpdate,
) -> Result<(), Box<dyn Error>> {
    if update.error.is_some()
        || !same_messages(
            &expected.message_list.messages,
            &update.surface.message_list.messages,
        )
    {
        return Err(format!(
            "Managed SelectHistory restore mismatch: error={:?}, expected={:?}, actual={:?}",
            update.error,
            surface_signature(&expected.message_list.messages),
            surface_signature(&update.surface.message_list.messages),
        )
        .into());
    }
    Ok(())
}

async fn wait_codex_ready(host: &ManagedChatHost) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        if let Some(update) = host.try_recv()?
            && codex_ready(&update)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("Codex did not become ready after Managed SelectHistory".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

fn codex_ready(update: &kcu_host_support::ManagedHostUpdate) -> bool {
    update
        .surface
        .vendor_bar
        .vendor_options
        .iter()
        .any(|option| option.id == CODEX)
}

fn same_messages(
    expected: &[katana_chat_ui::ChatUiMessageSurface],
    actual: &[katana_chat_ui::ChatUiMessageSurface],
) -> bool {
    expected == actual
}

fn require_empty_trace() -> Result<(), Box<dyn Error>> {
    let directory = std::env::var_os("KCU_ACP_RPC_TRACE_DIR")
        .ok_or("KCU_ACP_RPC_TRACE_DIR is required for the Managed history probe")?;
    if std::fs::read_dir(directory)?.next().is_some() {
        return Err("KCU_ACP_RPC_TRACE_DIR must be empty before the probe".into());
    }
    Ok(())
}

fn same_saved_snapshot(
    before: &katana_chat_ui::ChatSessionSnapshot,
    after: &katana_chat_ui::ChatSessionSnapshot,
) -> bool {
    before.version == after.version
        && before.title == after.title
        && before.messages == after.messages
        && before.outputs == after.outputs
        && before.draft == after.draft
        && before.context_usage == after.context_usage
        && before.account_usage == after.account_usage
        && before.next_message_id == after.next_message_id
        && before.next_output_id == after.next_output_id
        && same_saved_vendor_settings(&before.vendor_state, &after.vendor_state)
}

fn same_saved_vendor_settings(
    before: &katana_chat_ui::VendorUiState,
    after: &katana_chat_ui::VendorUiState,
) -> bool {
    // 接続時に再探索される候補一覧は保存値ではないため比較対象から除く。
    before.active_vendor_id == after.active_vendor_id
        && before.endpoint == after.endpoint
        && before.selected_model == after.selected_model
        && before.selected_mode == after.selected_mode
        && before.selected_thinking == after.selected_thinking
        && before.selected_permission == after.selected_permission
        && before.pending_permission == after.pending_permission
}

fn expected_surface(snapshot: &katana_chat_ui::ChatSessionSnapshot) -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.restore_snapshot(snapshot.clone());
    ChatUiSurface::from_render_model(&session.render_model())
}

fn surface_signature(messages: &[katana_chat_ui::ChatUiMessageSurface]) -> Vec<(u64, usize)> {
    messages
        .iter()
        .map(|message| (message.id, message.body.len()))
        .collect()
}

fn require_clean_trace() -> Result<(), Box<dyn Error>> {
    let Some(directory) = std::env::var_os("KCU_ACP_RPC_TRACE_DIR") else {
        return Err("KCU_ACP_RPC_TRACE_DIR is required for the Managed history probe".into());
    };
    let failed = std::fs::read_dir(directory)?.any(|entry| {
        entry
            .map(|entry| entry.file_name().to_string_lossy().ends_with(".failed"))
            .unwrap_or(true)
    });
    if failed {
        Err("ACP trace recorder reported a capture failure".into())
    } else {
        Ok(())
    }
}

fn require_trace_evidence() -> Result<(), Box<dyn Error>> {
    let methods = trace_methods()?;
    if methods
        .iter()
        .filter(|method| **method == "initialize")
        .count()
        < 1
        || methods
            .iter()
            .filter(|method| **method == "session/prompt")
            .count()
            != 3
    {
        return Err("ACP trace did not prove three explicit Codex prompts without resend".into());
    }
    Ok(())
}

fn trace_methods() -> Result<Vec<String>, Box<dyn Error>> {
    let directory = std::env::var_os("KCU_ACP_RPC_TRACE_DIR")
        .ok_or("KCU_ACP_RPC_TRACE_DIR is required for the Managed history probe")?;
    std::fs::read_dir(directory)?.try_fold(Vec::new(), |mut methods, entry| {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("codex-acp-")
        {
            let text = std::fs::read_to_string(entry.path())?;
            for line in text.lines() {
                let event = serde_json::from_str::<serde_json::Value>(line)?;
                methods.push(
                    event["method"]
                        .as_str()
                        .ok_or("ACP trace event has no method")?
                        .to_string(),
                );
            }
        }
        Ok::<_, Box<dyn Error>>(methods)
    })
}

async fn wait_for_ack(
    host: &ManagedChatHost,
    seq: u64,
) -> Result<kcu_host_support::ManagedHostUpdate, Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        if let Some(update) = host.try_recv()?
            && update.ack_seq >= seq
        {
            return Ok(update);
        }
        if Instant::now() >= deadline {
            return Err("Managed SelectHistory was not acknowledged before the deadline".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

async fn first_remote_turn(root: &Path, token: &str) -> Result<(), Box<dyn Error>> {
    let (mut host, mut providers) = ready_manager(root, false).await?;
    providers.select(&mut host, CODEX)?;
    let response = submit_and_wait(
        &mut providers,
        &mut host,
        &format!("Reply exactly {token}. Do not use tools or add any other text."),
    )
    .await?;
    require_contains(
        &response,
        token,
        "first ACP response did not preserve the token",
    )?;
    Ok(())
}

async fn restored_remote_turns(root: &Path, token: &str) -> Result<(), Box<dyn Error>> {
    let (mut host, mut providers) = ready_manager(root, true).await?;
    providers.select(&mut host, CODEX)?;
    providers.select(&mut host, CODEX)?;
    let restored = submit_and_wait(
        &mut providers,
        &mut host,
        "From the conversation history, return only the token that starts with KCU_HISTORY_.",
    )
    .await?;
    require_contains(&restored, token, "restored ACP session lost its history")?;
    providers.select(&mut host, CODEX)?;
    let continued = submit_and_wait(
        &mut providers,
        &mut host,
        "Reply exactly KCU_ACP_HISTORY_ACK. Do not mention an earlier token.",
    )
    .await?;
    require_ack(&continued, token)
}

async fn ready_manager(
    root: &Path,
    restored: bool,
) -> Result<(ChatHostState, ProviderManager), Box<dyn Error>> {
    let mut host = host(root)?;
    let mut providers = ProviderManager::new();
    providers.probe(root);
    if restored {
        providers.bind_session_identity(&mut host)?;
    }
    wait_ready(&mut providers, &mut host).await?;
    Ok((host, providers))
}

fn host(root: &Path) -> Result<ChatHostState, Box<dyn Error>> {
    Ok(ChatHostState::new(
        ChatSession::new(),
        ChatHistoryStore::new(root.into()),
        ChatOutputExecutor::new(root.into()),
    )?)
}

async fn wait_ready(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if host
            .session()
            .vendor_ui_state()
            .available_vendor_ids
            .iter()
            .any(|id| id == CODEX)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("Codex ACP did not become ready before the deadline".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

async fn submit_and_wait(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    prompt: &str,
) -> Result<String, Box<dyn Error>> {
    let first_new_message = host.session().snapshot().messages.len();
    host.session_mut().draft_mut().set_text(prompt);
    providers.submit(host).await?;
    wait_response(providers, host, first_new_message).await
}

async fn wait_response(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    first_new_message: usize,
) -> Result<String, Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if !host.session().render_model().input.can_cancel
            && let Some(message) = host
                .session()
                .snapshot()
                .messages
                .get(first_new_message + 1)
            && message.role == MessageRole::Assistant
        {
            return Ok(message.content.clone());
        }
        if Instant::now() >= deadline {
            return Err("Codex ACP prompt did not complete before the deadline".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

fn continuity_token() -> String {
    format!("KCU_HISTORY_{}_{}", std::process::id(), elapsed_nanos())
}

fn elapsed_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn require_contains(value: &str, token: &str, message: &str) -> Result<(), Box<dyn Error>> {
    if value.contains(token) {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn require_ack(value: &str, token: &str) -> Result<(), Box<dyn Error>> {
    if value.contains("KCU_ACP_HISTORY_ACK") && !value.contains(token) {
        Ok(())
    } else {
        Err("continued ACP prompt did not return the acknowledgement".into())
    }
}
