use std::{
    error::Error,
    path::PathBuf,
    time::{Duration, Instant},
};

use katana_chat_ui::{AgentActivityKind, ChatSession, MessageRole};
use kcu_host_support::{ChatHistoryStore, ChatHostState, ChatOutputExecutor, ProviderManager};
use tokio::{runtime::Builder, time};

const DEADLINE: Duration = Duration::from_secs(180);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const PROVIDERS: [&str; 4] = ["claude-code", "codex-cli", "github-copilot", "katanagent"];

fn main() -> Result<(), Box<dyn Error>> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let provider = provider_id()?;
    let root = tempfile::tempdir()?;
    let mut host = host(root.path())?;
    let mut providers = ProviderManager::new();
    providers.probe(root.path());
    wait_ready(&mut providers, &mut host, provider).await?;
    providers.select(&mut host, provider)?;
    select_required_model(&mut providers, &mut host, provider).await?;
    let token = format!("KCU_PROVIDER_PROBE_{}", std::process::id());
    let response = submit(&mut providers, &mut host, provider, &token).await?;
    if response.trim() == token {
        println!("ACP provider probe passed: {provider}");
        Ok(())
    } else {
        Err(format!("ACP provider {provider} returned an unexpected response: {response:?}").into())
    }
}

async fn select_required_model(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    provider: &str,
) -> Result<(), Box<dyn Error>> {
    if provider != "katanagent" {
        return Ok(());
    }
    let model = host
        .session()
        .vendor_ui_state()
        .model_options
        .first()
        .cloned()
        .ok_or("KatanAgent did not report an Ollama model")?;
    providers.select_model(host, &model).await?;
    Ok(())
}

fn provider_id() -> Result<&'static str, Box<dyn Error>> {
    let provider = std::env::args()
        .nth(1)
        .ok_or("usage: acp_provider_probe <provider-id>")?;
    PROVIDERS
        .into_iter()
        .find(|id| *id == provider)
        .ok_or_else(|| {
            format!(
                "unsupported provider {provider}; expected one of {}",
                PROVIDERS.join(", ")
            )
            .into()
        })
}

fn host(root: &std::path::Path) -> Result<ChatHostState, Box<dyn Error>> {
    Ok(ChatHostState::new(
        ChatSession::new(),
        ChatHistoryStore::new(PathBuf::from(root)),
        ChatOutputExecutor::new(PathBuf::from(root)),
    )?)
}

async fn wait_ready(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    provider: &str,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if host
            .session()
            .vendor_ui_state()
            .available_vendor_ids
            .iter()
            .any(|id| id == provider)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("ACP provider {provider} did not become ready").into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

async fn submit(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    provider: &str,
    token: &str,
) -> Result<String, Box<dyn Error>> {
    let count = host.session().snapshot().messages.len();
    host.session_mut()
        .draft_mut()
        .set_text(format!("Reply with exactly {token}"));
    providers.submit(host).await?;
    if provider == "katanagent" {
        assert_katanagent_activity_started(host, count)?;
    }
    wait_response(providers, host, count).await
}

fn assert_katanagent_activity_started(
    host: &ChatHostState,
    count: usize,
) -> Result<(), Box<dyn Error>> {
    let snapshot = host.session().snapshot();
    let assistant = snapshot
        .messages
        .get(count + 1)
        .ok_or("KatanAgent did not create an assistant activity message")?;
    if assistant.activity.as_ref().map(|activity| activity.kind)
        == Some(AgentActivityKind::Processing)
    {
        Ok(())
    } else {
        Err("KatanAgent did not expose Processing activity before its response".into())
    }
}

async fn wait_response(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    count: usize,
) -> Result<String, Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if !host.session().render_model().input.can_cancel
            && let Some(message) = host.session().snapshot().messages.get(count + 1)
            && message.role == MessageRole::Assistant
        {
            return completed_response(message);
        }
        if Instant::now() >= deadline {
            return Err("ACP provider prompt did not complete".into());
        }
        time::sleep(POLL_INTERVAL).await;
    }
}

fn completed_response(message: &katana_chat_ui::ChatMessage) -> Result<String, Box<dyn Error>> {
    if message.activity.is_some() {
        return Err("ACP provider left activity visible after completion".into());
    }
    match &message.status {
        katana_chat_ui::MessageStatus::Complete => Ok(message.content.clone()),
        katana_chat_ui::MessageStatus::Error(reason) => Err(reason.clone().into()),
        _ => Err("ACP provider stopped without a completed message".into()),
    }
}
