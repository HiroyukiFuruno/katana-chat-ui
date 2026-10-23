use katana_acp_client::protocol::{AcpStdioTransport, AcpStdioTransportConfig, AcpTransportEvent};
use std::{error::Error, time::Duration};

#[path = "acp_probe/prompt.rs"]
mod prompt;

fn main() -> Result<(), Box<dyn Error>> {
    tokio::runtime::Runtime::new()?.block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let command = arguments
        .next()
        .ok_or("usage: acp_probe <command> [args...]")?;
    let config = AcpStdioTransportConfig::new(command, std::env::current_dir()?).args(arguments);
    let mut transport = AcpStdioTransport::spawn(config)?;
    tokio::time::timeout(Duration::from_secs(90), handshake(&mut transport)).await??;
    if std::env::var_os("KCU_ACP_PROBE_PROMPTS").is_some() {
        tokio::time::timeout(Duration::from_secs(90), prompt::verify(&mut transport)).await??;
    }
    transport.close().await?;
    Ok(())
}

async fn handshake(transport: &mut AcpStdioTransport) -> Result<(), Box<dyn Error>> {
    while let Some(event) = transport.recv().await {
        match event {
            AcpTransportEvent::Initialized {
                protocol_version, ..
            } => {
                println!("initialized protocol={protocol_version}");
            }
            AcpTransportEvent::SessionStarted {
                session_id,
                config_ids,
                ..
            } => {
                println!("session={session_id} config_count={}", config_ids.len());
                return Ok(());
            }
            AcpTransportEvent::TransportError { message } => return Err(message.into()),
            AcpTransportEvent::Closed => {
                return Err("transport closed before session creation".into());
            }
            _ => {}
        }
    }
    Err("event stream ended before session creation".into())
}
