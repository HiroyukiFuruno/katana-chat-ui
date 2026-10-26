use katana_acp_client::protocol::{AcpStdioTransport, AcpStdioTransportConfig, AcpTransportEvent};
use katana_acp_client::{AcpContentBlock, AcpTextContent};
use kcu_host_support::acp_provider_profile;
use serde::Serialize;
use serde_json::Value;
use std::{error::Error, time::Duration};
use tokio::{runtime::Builder, time};

const DEADLINE: Duration = Duration::from_secs(90);

enum ProbeEvent {
    Started,
    Generating,
    Completed(String),
    Failed,
    Other,
}

fn main() -> Result<(), Box<dyn Error>> {
    Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let profile = acp_provider_profile("codex-cli", root.path())?;
    let config = AcpStdioTransportConfig::new(profile.command, root.path())
        .args(profile.args)
        .envs(profile.env);
    let mut transport = AcpStdioTransport::spawn(config)?;
    let result = time::timeout(DEADLINE, drive(&mut transport))
        .await
        .map_err(|_| "wire cancel timeout".into())
        .and_then(|value| value);
    let close_result = transport.close().await;
    match (result, close_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Ok(()), Ok(())) => Ok(()),
    }
}

async fn drive(transport: &mut AcpStdioTransport) -> Result<(), Box<dyn Error>> {
    let mut prompted = false;
    let mut cancelled = false;
    loop {
        match next_event(transport).await? {
            ProbeEvent::Started if !prompted => {
                prompted = true;
                send_prompt(transport).await?;
            }
            ProbeEvent::Generating if prompted && !cancelled => {
                transport.cancel().await?;
                cancelled = true;
                println!("generation_started=true cancel_sent=true");
            }
            ProbeEvent::Completed(reason) => return require_cancelled(cancelled, &reason),
            ProbeEvent::Failed => return Err("wire transport failed".into()),
            _ => {}
        }
    }
}

async fn next_event(transport: &mut AcpStdioTransport) -> Result<ProbeEvent, Box<dyn Error>> {
    let event = transport.recv().await.ok_or("wire transport closed")?;
    Ok(match event {
        AcpTransportEvent::SessionStarted { .. } => ProbeEvent::Started,
        AcpTransportEvent::SessionUpdate { update, .. } => {
            if update_is_generating(&update)? {
                ProbeEvent::Generating
            } else {
                ProbeEvent::Other
            }
        }
        AcpTransportEvent::PromptCompleted { stop_reason, .. } => {
            ProbeEvent::Completed(stop_reason)
        }
        AcpTransportEvent::TransportError { .. }
        | AcpTransportEvent::CommandRejected { .. }
        | AcpTransportEvent::Closed => ProbeEvent::Failed,
        _ => ProbeEvent::Other,
    })
}

fn update_is_generating(update: &impl Serialize) -> Result<bool, Box<dyn Error>> {
    // 標準ACPの種別だけを判定し、chunk本文はログや証跡へ保存しない。
    let value = serde_json::to_value(update)?;
    Ok(matches!(
        value.get("sessionUpdate").and_then(Value::as_str),
        Some("agent_message_chunk" | "agent_thought_chunk")
    ))
}

async fn send_prompt(transport: &AcpStdioTransport) -> Result<(), Box<dyn Error>> {
    // 生成開始前の取消競合と区別するため、実chunkを受けてから取消す。
    transport.prompt(vec![AcpContentBlock::Text(AcpTextContent::new(
        "Write a detailed explanation of how a compiler works. Do not use tools or access files.",
    ))]).await?;
    Ok(())
}

fn require_cancelled(cancel_sent: bool, reason: &str) -> Result<(), Box<dyn Error>> {
    // SDKの実PromptResponseを既存transportがDebug変換した値で、raw JSONの保存ではない。
    println!("SDK stop reason={reason}");
    if cancel_sent && reason == "Cancelled" {
        Ok(())
    } else {
        Err("active-generation cancellation was not confirmed".into())
    }
}
