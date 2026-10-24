use agent_client_protocol::schema::v1::{ContentBlock, SessionUpdate, ToolKind};
use katana_acp_client::protocol::{
    AcpPermissionContext, AcpPermissionOption, AcpSessionConfig, AcpSessionConfigCategory,
    AcpTransportEvent,
};
use katana_chat_ui::AgentActivityKind;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum AcpProviderProjection {
    Connecting,
    Unchanged,
    Ready {
        models: Vec<AcpProviderModel>,
    },
    Text(String),
    Thinking(String),
    Activity(AgentActivityKind),
    Finished,
    Permission {
        request_id: u64,
        context: AcpPermissionContext,
        options: Vec<AcpPermissionOption>,
    },
    ConfigRejected {
        config_id: String,
        message: String,
    },
    ConfigUpdated {
        config_id: String,
    },
    Unavailable(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpProviderModel {
    pub config_id: String,
    pub id: String,
    pub name: String,
    pub selected: bool,
}

pub(super) fn project(event: AcpTransportEvent) -> AcpProviderProjection {
    match event {
        AcpTransportEvent::Initialized { .. } => AcpProviderProjection::Connecting,
        AcpTransportEvent::SessionStarted { config_options, .. }
        | AcpTransportEvent::SessionConfigOptionsUpdated { config_options, .. } => {
            AcpProviderProjection::Ready {
                models: models(config_options),
            }
        }
        AcpTransportEvent::SessionUpdate { update, .. } => project_update(*update),
        AcpTransportEvent::PromptCompleted { .. } => AcpProviderProjection::Finished,
        AcpTransportEvent::PermissionRequested {
            request_id,
            context,
            options,
            ..
        } => project_permission(request_id, context, options),
        AcpTransportEvent::ConfigRejected { config_id, message } => {
            AcpProviderProjection::ConfigRejected { config_id, message }
        }
        AcpTransportEvent::TransportError { message }
        | AcpTransportEvent::CommandRejected {
            reason: message, ..
        } => AcpProviderProjection::Failed(message),
        AcpTransportEvent::ConfigUpdated { config_id } => {
            AcpProviderProjection::ConfigUpdated { config_id }
        }
        AcpTransportEvent::Closed => {
            AcpProviderProjection::Unavailable("ACP provider process closed".to_string())
        }
    }
}

fn project_permission(
    request_id: u64,
    context: AcpPermissionContext,
    options: Vec<AcpPermissionOption>,
) -> AcpProviderProjection {
    AcpProviderProjection::Permission {
        request_id,
        context,
        options,
    }
}

fn models(options: Vec<AcpSessionConfig>) -> Vec<AcpProviderModel> {
    options
        .into_iter()
        .filter(|option| option.category == AcpSessionConfigCategory::Model)
        .flat_map(|option| {
            option
                .values
                .into_iter()
                .map(move |value| AcpProviderModel {
                    config_id: option.id.clone(),
                    selected: option.current_value.as_deref() == Some(value.id.as_str()),
                    id: value.id,
                    name: value.name,
                })
        })
        .collect()
}

fn project_update(update: SessionUpdate) -> AcpProviderProjection {
    match update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            content(chunk.content, AcpProviderProjection::Text)
        }
        SessionUpdate::AgentThoughtChunk(chunk) => {
            content(chunk.content, AcpProviderProjection::Thinking)
        }
        SessionUpdate::ToolCall(call) => activity(call.kind),
        SessionUpdate::ToolCallUpdate(update) => update_activity(update.fields.kind),
        SessionUpdate::Plan(_) => processing(),
        _ => AcpProviderProjection::Unchanged,
    }
}

fn content(
    content: ContentBlock,
    projection: impl FnOnce(String) -> AcpProviderProjection,
) -> AcpProviderProjection {
    match content {
        ContentBlock::Text(text) => projection(text.text),
        _ => processing(),
    }
}

fn activity(kind: ToolKind) -> AcpProviderProjection {
    AcpProviderProjection::Activity(activity_kind(kind))
}

fn update_activity(kind: Option<ToolKind>) -> AcpProviderProjection {
    match kind {
        Some(kind) => activity(kind),
        None => AcpProviderProjection::Unchanged,
    }
}

fn processing() -> AcpProviderProjection {
    AcpProviderProjection::Activity(AgentActivityKind::Processing)
}

fn activity_kind(kind: ToolKind) -> AgentActivityKind {
    match kind {
        ToolKind::Read => AgentActivityKind::Reading,
        ToolKind::Edit | ToolKind::Delete | ToolKind::Move => AgentActivityKind::Editing,
        ToolKind::Search => AgentActivityKind::Searching,
        ToolKind::Fetch => AgentActivityKind::WebSearching,
        ToolKind::Execute => AgentActivityKind::Executing,
        _ => AgentActivityKind::Processing,
    }
}
