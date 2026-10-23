use katana_chat_ui::ChatUiAction;

use crate::{AcpPermissionDecision, ChatHostEffect, ChatHostError, ChatHostState, ProviderManager};

use super::super::ManagedHostCommand;

pub(super) async fn dispatch(
    command: ManagedHostCommand,
    host: &mut ChatHostState,
    providers: &mut ProviderManager,
) -> Result<Option<ChatHostEffect>, ChatHostError> {
    match command {
        ManagedHostCommand::DraftChanged(text) => {
            host.session_mut().draft_mut().set_text(text);
            Ok(None)
        }
        ManagedHostCommand::AddAttachment(attachment) => {
            host.session_mut().draft_mut().add_attachment(attachment);
            host.persist()?;
            Ok(None)
        }
        ManagedHostCommand::Action(action) => dispatch_action(action, host, providers).await,
    }
}

async fn dispatch_action(
    action: ChatUiAction,
    host: &mut ChatHostState,
    providers: &mut ProviderManager,
) -> Result<Option<ChatHostEffect>, ChatHostError> {
    match action {
        ChatUiAction::Submit => providers.submit(host).await?,
        ChatUiAction::Stop => providers.stop(host).await?,
        ChatUiAction::SelectVendor { vendor_id } => providers.select(host, &vendor_id)?,
        ChatUiAction::SelectVendorControl { key, value } if key == "model" => {
            providers.select_model(host, &value).await?;
            providers.poll(host).await?;
        }
        ChatUiAction::RespondPermission {
            request_id,
            option_id,
        } => {
            providers
                .respond_permission(host, request_id, permission_decision(option_id))
                .await?;
        }
        action @ (ChatUiAction::NewChat | ChatUiAction::SelectHistory { .. }) => {
            validate_boundary_action(host, &action)?;
            providers.begin_session_boundary(host).await?;
            let effect = host.dispatch(action)?;
            providers.bind_session_identity(host)?;
            return Ok(Some(effect));
        }
        action => return host.dispatch(action).map(Some),
    }
    Ok(None)
}

fn validate_boundary_action(
    host: &mut ChatHostState,
    action: &ChatUiAction,
) -> Result<(), ChatHostError> {
    let ChatUiAction::SelectHistory { session_id } = action else {
        return Ok(());
    };
    let (record, issues) = host
        .history
        .load_with_diagnostics(session_id)
        .map_err(ChatHostError::History)?;
    host.remember_history_issues_for_boundary(issues);
    record
        .map(|_| ())
        .ok_or_else(|| ChatHostError::History(format!("session is not found: {session_id}")))
}

fn permission_decision(option_id: Option<String>) -> AcpPermissionDecision {
    match option_id {
        Some(option_id) => AcpPermissionDecision::Selected { option_id },
        None => AcpPermissionDecision::Cancelled,
    }
}

#[cfg(test)]
#[path = "dispatch/tests.rs"]
mod tests;
