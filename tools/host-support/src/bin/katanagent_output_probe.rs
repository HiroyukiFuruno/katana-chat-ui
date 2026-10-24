use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use katana_chat_ui::{
    ChatOutput, ChatOutputKind, ChatSession, ChatUiAction, HostActionKind, MessageRole,
    MessageStatus,
};
use kcu_host_support::{ChatHistoryStore, ChatHostState, ChatOutputExecutor, ProviderManager};
use tokio::{runtime::Builder, time};

const KATANAGENT: &str = "katanagent";
const DEADLINE: Duration = Duration::from_secs(180);
const POLL_INTERVAL: Duration = Duration::from_millis(25);

fn main() -> Result<(), Box<dyn Error>> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let mut host = host(root.path())?;
    let mut providers = ProviderManager::new();
    providers.probe(root.path());
    wait_ready(&mut providers, &mut host).await?;
    providers.select(&mut host, KATANAGENT)?;
    select_model(&mut providers, &mut host).await?;

    let token = format!("KCU_KATAN_OUTPUT_{}", std::process::id());
    let created = create_markdown(&mut providers, &mut host, &token).await?;
    let target = root.path().join(&created.path);
    apply_create(&mut host, &created, &target)?;

    let edited = edit_markdown(&mut providers, &mut host, &created, &target, &token).await?;
    apply_and_undo_edit(&mut host, &edited, &target)?;

    println!("KatanAgent Markdown output probe passed");
    Ok(())
}

fn host(root: &Path) -> Result<ChatHostState, Box<dyn Error>> {
    Ok(ChatHostState::new(
        ChatSession::new(),
        ChatHistoryStore::new(PathBuf::from(root)),
        ChatOutputExecutor::new(PathBuf::from(root)),
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
            .any(|id| id == KATANAGENT)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("KatanAgent did not become ready".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

async fn select_model(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    let model = host
        .session()
        .vendor_ui_state()
        .model_options
        .first()
        .cloned()
        .ok_or("KatanAgent did not report an Ollama model")?;
    providers.select_model(host, &model).await?;
    providers.poll(host).await?;
    Ok(())
}

async fn create_markdown(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    token: &str,
) -> Result<katana_chat_ui::FileCandidateOutput, Box<dyn Error>> {
    let path = format!("tmp/{token}.md");
    host.session_mut().draft_mut().set_text(format!(
        "{path} を作成してください。Markdown本文だけを返してください。1行目は # {token}、空行の後に {token} body を書いてください。"
    ));
    let output = submit_and_wait(providers, host).await?;
    let ChatOutputKind::FileCandidate(file) = output.kind else {
        return Err("KatanAgent create did not produce a Markdown file candidate".into());
    };
    if file.path != path || !file.content.contains(token) {
        return Err(
            "KatanAgent create candidate did not preserve its requested target or content".into(),
        );
    }
    Ok(file)
}

fn apply_create(
    host: &mut ChatHostState,
    created: &katana_chat_ui::FileCandidateOutput,
    target: &Path,
) -> Result<(), Box<dyn Error>> {
    let output_id = output_id(
        host,
        |kind| matches!(kind, ChatOutputKind::FileCandidate(file) if file == created),
    )?;
    host.dispatch(output_action(output_id, HostActionKind::CreateFile))?;
    let content = fs::read_to_string(target)?;
    if content == created.content {
        Ok(())
    } else {
        Err("Create file did not write the generated candidate".into())
    }
}

async fn edit_markdown(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    created: &katana_chat_ui::FileCandidateOutput,
    target: &Path,
    token: &str,
) -> Result<katana_chat_ui::DiffCandidateOutput, Box<dyn Error>> {
    let original = fs::read_to_string(target)?;
    if !host.session().draft().attachments.is_empty() {
        return Err("text-only Markdown edit unexpectedly has an attachment".into());
    }
    host.session_mut().draft_mut().set_text(format!(
        "{} を編集してください。先頭見出しを # {token} EDITED に変え、本文の {token} body は残してください。Markdown本文だけを返してください。", created.path
    ));
    let output = submit_and_wait(providers, host).await?;
    let ChatOutputKind::DiffCandidate(diff) = output.kind else {
        return Err("KatanAgent edit did not produce a Markdown diff candidate".into());
    };
    if diff.target_path != created.path
        || diff.original_content != original
        || diff.updated_content == original
        || !diff.updated_content.contains(&format!("{token} EDITED"))
    {
        return Err(
            "KatanAgent edit candidate did not produce the requested Markdown change".into(),
        );
    }
    Ok(diff)
}

fn apply_and_undo_edit(
    host: &mut ChatHostState,
    edited: &katana_chat_ui::DiffCandidateOutput,
    target: &Path,
) -> Result<(), Box<dyn Error>> {
    let output_id = output_id(
        host,
        |kind| matches!(kind, ChatOutputKind::DiffCandidate(diff) if diff == edited),
    )?;
    host.dispatch(output_action(output_id, HostActionKind::ApplyDiff))?;
    if fs::read_to_string(target)? != edited.updated_content {
        return Err("Apply diff did not write the generated edit".into());
    }
    host.dispatch(output_action(output_id, HostActionKind::UndoChange))?;
    if fs::read_to_string(target)? == edited.original_content {
        Ok(())
    } else {
        Err("Undo change did not restore the original Markdown".into())
    }
}

async fn submit_and_wait(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<ChatOutput, Box<dyn Error>> {
    let output_count = host.session().snapshot().outputs.len();
    let message_count = host.session().snapshot().messages.len();
    providers.submit(host).await?;
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        let snapshot = host.session().snapshot();
        if !host.session().render_model().input.can_cancel
            && let Some(message) = snapshot.messages.get(message_count + 1)
            && message.role == MessageRole::Assistant
        {
            return match &message.status {
                MessageStatus::Complete => snapshot
                    .outputs
                    .get(output_count)
                    .cloned()
                    .ok_or_else(|| "KatanAgent completed without a Markdown candidate".into()),
                MessageStatus::Error(reason) => Err(reason.clone().into()),
                _ => Err("KatanAgent stopped without completing its response".into()),
            };
        }
        if Instant::now() >= deadline {
            return Err("KatanAgent Markdown prompt did not complete".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

fn output_id(
    host: &ChatHostState,
    matches_output: impl Fn(&ChatOutputKind) -> bool,
) -> Result<u64, Box<dyn Error>> {
    host.session()
        .snapshot()
        .outputs
        .iter()
        .find(|output| matches_output(&output.kind))
        .map(|output| output.id)
        .ok_or_else(|| "generated Markdown output is missing".into())
}

fn output_action(output_id: u64, kind: HostActionKind) -> ChatUiAction {
    ChatUiAction::Output { output_id, kind }
}
