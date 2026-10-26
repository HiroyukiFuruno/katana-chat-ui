use katana_chat_ui::{
    ChatSession, ChatSessionSnapshot, MessageRole, MessageStatus, VendorUiPermissionRequest,
};
use kcu_host_support::{
    AcpPermissionDecision, ChatHistoryStore, ChatHostState, ChatOutputExecutor, ProviderManager,
};
use std::{
    error::Error,
    path::Path,
    time::{Duration, Instant},
};
use tokio::{runtime::Builder, time};

const CLAUDE: &str = "claude-code";
const CODEX: &str = "codex-cli";
const KATAN: &str = "katanagent";
const CLAUDE_REJECT_OPTION_ID: &str = "reject";
const DEADLINE: Duration = Duration::from_secs(90);
const POLL: Duration = Duration::from_millis(25);

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("permission-cancel|permission-deny|codex-permission-cancel|katan-model")?;
    validate_mode(&mode)?;
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run(&mode))
}

fn validate_mode(mode: &str) -> Result<(), Box<dyn Error>> {
    if mode == "codex-permission-cancel"
        && std::env::var("INITIAL_AGENT_MODE").as_deref() != Ok("read-only")
    {
        return Err("Codex permission probe requires explicit read-only mode".into());
    }
    if !matches!(
        mode,
        "permission-cancel" | "permission-deny" | "codex-permission-cancel" | "katan-model"
    ) {
        return Err("unsupported mode".into());
    }
    Ok(())
}

async fn run_case(
    mode: &str,
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    marker: &Path,
) -> Result<(), Box<dyn Error>> {
    match mode {
        "permission-cancel" => permission_cancel(providers, host, marker, CLAUDE).await,
        "codex-permission-cancel" => permission_cancel(providers, host, marker, CODEX).await,
        "permission-deny" => permission_deny(providers, host, marker).await,
        "katan-model" => katan_model(providers, host).await,
        _ => Err("unsupported mode".into()),
    }
}

async fn run(mode: &str) -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let root_path = root.path().to_path_buf();
    let marker = root_path.join(format!("tmp/KCU_PERMISSION_MARKER_{}", std::process::id()));
    std::fs::create_dir_all(marker.parent().ok_or("marker parent missing")?)?;
    let before = marker.exists();
    if before {
        return Err("marker unexpectedly existed before probe".into());
    }
    let mut host = host(&root_path)?;
    let mut providers = ProviderManager::new();
    providers.probe(&root_path);
    let result = run_case(mode, &mut providers, &mut host, &marker).await;
    let after = marker.exists();
    println!("marker_before={before} marker_after={after}");
    if after {
        return Err("probe created the marker unexpectedly".into());
    }
    result
}

fn host(root: &Path) -> Result<ChatHostState, Box<dyn Error>> {
    Ok(ChatHostState::new(
        ChatSession::new(),
        ChatHistoryStore::new(root.to_path_buf()),
        ChatOutputExecutor::new(root.to_path_buf()),
    )?)
}

async fn ready(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    vendor: &str,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if host
            .session()
            .vendor_ui_state()
            .available_vendor_ids
            .iter()
            .any(|id| id == vendor)
        {
            providers.select(host, vendor)?;
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("{vendor} unavailable").into());
        }
        time::sleep(POLL).await;
    }
}

async fn request_marker_permission(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    marker: &Path,
    vendor: &str,
) -> Result<VendorUiPermissionRequest, Box<dyn Error>> {
    ready(providers, host, vendor).await?;
    println!("provider_ready=true");
    // 書込toolを明示し、説明だけの応答と実permission往復を区別する。
    let operation = if vendor == CLAUDE {
        "Use the Write tool exactly once to create this marker with content KCU_PERMISSION_MARKER:"
    } else {
        "Attempt only creating this marker in the session workspace:"
    };
    host.session_mut().draft_mut().set_text(format!(
        "{operation} {}. Do not use another tool or path. If rejected or cancelled, stop without retrying.", marker.display()
    ));
    providers.submit(host).await?;
    println!("prompt_submitted=true");
    let request = wait_permission(providers, host).await?;
    assert_permission_options(&request)?;
    Ok(request)
}

async fn permission_cancel(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    marker: &Path,
    vendor: &str,
) -> Result<(), Box<dyn Error>> {
    let request = request_marker_permission(providers, host, marker, vendor).await?;
    println!(
        "permission request: options={} title={} details={}",
        request.options.len(),
        request.title.is_some(),
        request.details.len()
    );
    host.session_mut()
        .draft_mut()
        .set_text("KCU_PERMISSION_NEXT_DRAFT");
    let before_decision = host.session().snapshot();
    providers
        .respond_permission(host, request.request_id, AcpPermissionDecision::Cancelled)
        .await?;
    wait_terminal(providers, host).await?;
    assert_permission_terminal(host, &before_decision, "cancel")?;
    Ok(())
}

fn assert_permission_options(request: &VendorUiPermissionRequest) -> Result<(), Box<dyn Error>> {
    if request.options.is_empty() {
        return Err("permission request advertised no options".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    if request
        .options
        .iter()
        .any(|option| !ids.insert(option.id.as_str()))
    {
        return Err("permission request advertised duplicate option IDs".into());
    }
    Ok(())
}

fn assert_permission_terminal(
    host: &ChatHostState,
    before: &ChatSessionSnapshot,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let after = host.session().snapshot();
    let last = after
        .messages
        .last()
        .ok_or("permission case has no assistant message")?;
    if after.messages.len() != before.messages.len() || after.draft != before.draft {
        return Err(format!("permission {label} changed messages or next draft").into());
    }
    if last.role != MessageRole::Assistant
        || !matches!(last.status, MessageStatus::Complete)
        || host
            .session()
            .vendor_ui_state()
            .pending_permission
            .is_some()
    {
        return Err(format!(
            "permission {label} did not reach Complete with no pending permission"
        )
        .into());
    }
    Ok(())
}

async fn wait_permission(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<VendorUiPermissionRequest, Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if let Some(request) = host.session().vendor_ui_state().pending_permission.clone() {
            return Ok(request);
        }
        if !host.session().render_model().input.can_cancel {
            return Err("prompt ended without a permission request".into());
        }
        if Instant::now() >= deadline {
            return Err("permission request not observed".into());
        }
        time::sleep(POLL).await;
    }
}

async fn permission_deny(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    marker: &Path,
) -> Result<(), Box<dyn Error>> {
    let request = request_marker_permission(providers, host, marker, CLAUDE).await?;
    let option = request
        .options
        .iter()
        .find(|option| option.id == CLAUDE_REJECT_OPTION_ID)
        .ok_or("Claude ACP did not advertise the exact reject option")?;
    println!(
        "permission deny request: options={} reject_id_present=true",
        request.options.len()
    );
    host.session_mut()
        .draft_mut()
        .set_text("KCU_PERMISSION_NEXT_DRAFT");
    let before_decision = host.session().snapshot();
    providers
        .respond_permission(
            host,
            request.request_id,
            AcpPermissionDecision::Selected {
                option_id: option.id.clone(),
            },
        )
        .await?;
    wait_terminal(providers, host).await?;
    assert_permission_terminal(host, &before_decision, "deny")?;
    Ok(())
}

async fn wait_terminal(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if !host.session().render_model().input.can_cancel
            && host
                .session()
                .vendor_ui_state()
                .pending_permission
                .is_none()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("cancel did not reach terminal state".into());
        }
        time::sleep(POLL).await;
    }
}

async fn katan_model(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
) -> Result<(), Box<dyn Error>> {
    ready(providers, host, KATAN).await?;
    let models = host.session().vendor_ui_state().model_options.clone();
    if models.len() < 2 {
        return Err(format!("advertised model count={} (need >=2)", models.len()).into());
    }
    // 新しい検証sessionの初期モデルを明示し、catalog取得だけの暗黙選択とは区別する。
    let original = models[0].clone();
    providers.select_model(host, &original).await?;
    wait_model(providers, host, &original).await?;
    let second = &models[1];
    for model in [second, &original] {
        providers.select_model(host, model).await?;
        wait_model(providers, host, model).await?;
    }
    println!(
        "KatanAgent explicit initial selection/switch/restore host checks passed; options={}",
        models.len()
    );
    Ok(())
}

async fn wait_model(
    providers: &mut ProviderManager,
    host: &mut ChatHostState,
    model: &str,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        providers.poll(host).await?;
        if host.session().vendor_ui_state().selected_model.as_deref() == Some(model) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("model confirmation timeout".into());
        }
        time::sleep(POLL).await;
    }
}

// Claude ACP 0.96の公式reject IDだけを使い、ラベルから拒否optionを推測しない。
