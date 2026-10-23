use std::{error::Error, path::Path, time::Duration};

use katana_chat_ui::{ChatSession, ChatUiAction, MessageStatus};
use kcu_host_support::{ChatHistoryStore, ChatHostState, ChatOutputExecutor, ProviderManager};
use tokio::{runtime::Builder, time};

const CODEX: &str = "codex-cli";
const DEADLINE: Duration = Duration::from_secs(90);

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
    providers.select(&mut host, CODEX)?;
    cancel_turn(&mut providers, &mut host).await?;
    resume_turn(&mut providers, &mut host).await?;
    println!(
        "ACP cancellation probe passed: stop pending, draft preserved, terminal received, next prompt completed"
    );
    Ok(())
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
    time::timeout(DEADLINE, async {
        let mut poll = time::interval(Duration::from_millis(25));
        loop {
            poll.tick().await;
            providers.poll(host).await?;
            if host
                .session()
                .vendor_ui_state()
                .available_vendor_ids
                .iter()
                .any(|id| id == CODEX)
            {
                return Ok::<(), Box<dyn Error>>(());
            }
        }
    })
    .await?
}

async fn cancel_turn(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    host.session_mut()
        .draft_mut()
        .set_text("Write a detailed explanation of how a compiler works.");
    providers.submit(host).await?;
    providers.stop(host).await?;
    providers.stop(host).await?;
    host.session_mut()
        .draft_mut()
        .set_text("Reply with exactly KCU_CANCEL_RESUMED");
    let before = host.session().clone();
    if !host.session().render_model().input.can_cancel {
        return Err("Stop prematurely completed the outstanding prompt".into());
    }
    if providers.submit(host).await.is_ok() || host.dispatch(ChatUiAction::Submit).is_ok() {
        return Err("A submission was accepted before cancellation completed".into());
    }
    if host.session() != &before {
        return Err("Rejected submission changed the draft or live conversation".into());
    }
    wait_complete(providers, host).await?;
    require_cancelled_turn(host, &before)
}

fn require_cancelled_turn(
    host: &ChatHostState,
    before: &ChatSession,
) -> Result<(), Box<dyn Error>> {
    let actual = host.session().snapshot();
    let expected = before.snapshot();
    if actual.messages.len() != expected.messages.len()
        || actual.draft != expected.draft
        || actual.messages.last().map(|m| &m.content)
            != expected.messages.last().map(|m| &m.content)
    {
        return Err("Residual cancellation updates changed messages or the next draft".into());
    }
    if !matches!(
        actual.messages.last().map(|m| &m.status),
        Some(MessageStatus::Complete)
    ) {
        return Err("Cancelled prompt ended as a provider failure".into());
    }
    Ok(())
}

async fn resume_turn(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    providers.submit(host).await?;
    wait_complete(providers, host).await?;
    let snapshot = host.session().snapshot();
    let response = snapshot.messages.last().ok_or("Missing resumed response")?;
    if response.status != MessageStatus::Complete || response.content.trim() != "KCU_CANCEL_RESUMED"
    {
        return Err("The next prompt did not complete after cancellation".into());
    }
    Ok(())
}

async fn wait_complete(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    time::timeout(DEADLINE, async {
        let mut poll = time::interval(Duration::from_millis(25));
        while host.session().render_model().input.can_cancel {
            poll.tick().await;
            providers.poll(host).await?;
        }
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}
