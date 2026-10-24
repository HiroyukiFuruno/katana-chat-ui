use agent_client_protocol::schema::v1::{ContentBlock, SessionUpdate};
use katana_acp_client::{
    AcpContentBlock,
    protocol::{AcpPermissionDecision, AcpStdioTransport, AcpTransportEvent},
};
use std::error::Error;

pub(super) async fn verify(transport: &mut AcpStdioTransport) -> Result<(), Box<dyn Error>> {
    let first = turn(transport, "Reply exactly KCU_ACP_OK. Do not use any tools.").await?;
    let second = turn(
        transport,
        "Repeat your previous reply exactly. Do not use any tools.",
    )
    .await?;
    if first != second {
        return Err("consecutive prompts changed the ACP session".into());
    }
    println!("two streamed prompts passed in the same session");
    Ok(())
}

async fn turn(transport: &mut AcpStdioTransport, text: &str) -> Result<String, Box<dyn Error>> {
    transport
        .prompt(vec![AcpContentBlock::Text(
            katana_acp_client::AcpTextContent { text: text.into() },
        )])
        .await?;
    receive_turn(transport).await
}

async fn receive_turn(transport: &mut AcpStdioTransport) -> Result<String, Box<dyn Error>> {
    let mut body = String::new();
    while let Some(event) = transport.recv().await {
        match event {
            AcpTransportEvent::SessionUpdate { update, .. } => {
                if let SessionUpdate::AgentMessageChunk(chunk) = *update
                    && let ContentBlock::Text(text) = chunk.content
                {
                    body.push_str(&text.text);
                }
            }
            AcpTransportEvent::PromptCompleted { session_id, .. } => {
                validate_reply(&body)?;
                return Ok(session_id);
            }
            AcpTransportEvent::PermissionRequested { request_id, .. } => {
                transport
                    .respond_permission(request_id, AcpPermissionDecision::Cancelled)
                    .await?;
            }
            AcpTransportEvent::TransportError { message } => return Err(message.into()),
            AcpTransportEvent::Closed => {
                return Err("transport closed before prompt completion".into());
            }
            _ => {}
        }
    }
    Err("event stream ended before prompt completion".into())
}

fn validate_reply(body: &str) -> Result<(), Box<dyn Error>> {
    // ACP agent は回答前に診断通知も text chunk として流すため、回答の終端を検証する。
    if body.trim().lines().last() != Some("KCU_ACP_OK") {
        return Err(format!("unexpected streamed response: {body:?}").into());
    }
    Ok(())
}
