use super::types::{
    AcpPermissionContext, AcpPermissionDecision, AcpPermissionOption, AcpTransportEvent,
};
use agent_client_protocol::schema::v1::{
    PermissionOption, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome, ToolCallContent, ToolCallUpdateFields,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot;

#[derive(Clone, Default)]
pub(crate) struct PendingPermissions {
    state: Arc<Mutex<PendingPermissionState>>,
}

#[derive(Default)]
struct PendingPermissionState {
    cancelling: bool,
    requests: BTreeMap<u64, oneshot::Sender<AcpPermissionDecision>>,
}

pub(crate) enum PermissionRegistration {
    Registered,
    Cancelled,
}

pub(crate) fn permission_event(
    request_id: u64,
    request: &RequestPermissionRequest,
) -> AcpTransportEvent {
    let options = request
        .options
        .iter()
        .map(|option| AcpPermissionOption {
            option_id: option.option_id.to_string(),
            name: option.name.clone(),
        })
        .collect();
    let fields = &request.tool_call.fields;
    AcpTransportEvent::PermissionRequested {
        request_id,
        session_id: request.session_id.to_string(),
        context: AcpPermissionContext {
            tool_call_id: request.tool_call.tool_call_id.to_string(),
            title: fields.title.clone(),
            details: permission_details(fields),
        },
        options,
    }
}

fn permission_details(fields: &ToolCallUpdateFields) -> Vec<String> {
    let mut details = Vec::new();
    if let Some(name) = &fields.name {
        details.push(format!("ツール名: {name}"));
    }
    if let Some(content) = &fields.content {
        details.extend(content.iter().map(permission_content_detail));
    }
    if let Some(locations) = &fields.locations {
        for location in locations {
            let line = location
                .line
                .map_or_else(String::new, |line| format!(":{line}"));
            details.push(format!("対象位置: {}{line}", location.path.display()));
        }
    }
    if let Some(raw_input) = &fields.raw_input {
        details.push(format!(
            "提供された入力:\n{}",
            serde_json::to_string_pretty(raw_input)
                .unwrap_or_else(|_| "入力を表示できません".to_string())
        ));
    }
    details
}

fn permission_content_detail(item: &ToolCallContent) -> String {
    match item {
        ToolCallContent::Content(content) => match &content.content {
            agent_client_protocol::schema::v1::ContentBlock::Text(text) => {
                format!("提供された内容:\n{}", text.text)
            }
            block => format!(
                "提供された内容:\n{}",
                serde_json::to_string_pretty(block)
                    .unwrap_or_else(|_| "内容を表示できません".to_string())
            ),
        },
        ToolCallContent::Diff(diff) => format!(
            "変更内容: {}\n変更前:\n{}\n変更後:\n{}",
            diff.path.display(),
            diff.old_text.as_deref().unwrap_or("(新規ファイル)"),
            diff.new_text
        ),
        ToolCallContent::Terminal(terminal) => format!(
            "提供された端末情報:\n{}",
            serde_json::to_string_pretty(terminal)
                .unwrap_or_else(|_| "端末情報を表示できません".to_string())
        ),
        _ => format!(
            "提供された内容:\n{}",
            serde_json::to_string_pretty(item)
                .unwrap_or_else(|_| "内容を表示できません".to_string())
        ),
    }
}

pub(crate) fn permission_response(
    decision: AcpPermissionDecision,
    options: &[PermissionOption],
) -> RequestPermissionResponse {
    let outcome = match decision {
        AcpPermissionDecision::Cancelled => RequestPermissionOutcome::Cancelled,
        AcpPermissionDecision::Selected { option_id }
            if options
                .iter()
                .any(|option| option.option_id.to_string() == option_id) =>
        {
            RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(option_id))
        }
        AcpPermissionDecision::Selected { .. } => RequestPermissionOutcome::Cancelled,
    };
    RequestPermissionResponse::new(outcome)
}

pub(crate) fn insert_permission(
    pending: &PendingPermissions,
    request_id: u64,
    sender: oneshot::Sender<AcpPermissionDecision>,
) -> Result<PermissionRegistration, agent_client_protocol::Error> {
    let mut state = pending.state.lock().map_err(|_| {
        agent_client_protocol::Error::internal_error().data("permission state poisoned")
    })?;
    if state.cancelling {
        let _ = sender.send(AcpPermissionDecision::Cancelled);
        return Ok(PermissionRegistration::Cancelled);
    }
    state.requests.insert(request_id, sender);
    Ok(PermissionRegistration::Registered)
}

pub(crate) fn resolve_permission(
    pending: &PendingPermissions,
    request_id: u64,
    decision: AcpPermissionDecision,
) -> Result<(), String> {
    let sender = pending
        .state
        .lock()
        .map_err(|_| "permission state poisoned".to_string())?
        .requests
        .remove(&request_id);
    sender
        .ok_or_else(|| "permission request is no longer pending".to_string())?
        .send(decision)
        .map_err(|_| "permission response receiver closed".to_string())
}

pub(crate) fn cancel_permissions(pending: &PendingPermissions) {
    if let Ok(mut state) = pending.state.lock() {
        state.cancelling = true;
        for (_, sender) in std::mem::take(&mut state.requests) {
            let _ = sender.send(AcpPermissionDecision::Cancelled);
        }
    }
}

pub(crate) fn reset_permission_cancellation(pending: &PendingPermissions) -> Result<(), String> {
    let mut state = pending
        .state
        .lock()
        .map_err(|_| "permission state poisoned".to_string())?;
    state.cancelling = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AcpPermissionContext, PendingPermissions, permission_event, permission_response};
    use crate::protocol::transport::AcpPermissionDecision;
    use agent_client_protocol::schema::v1::{
        PermissionOption, PermissionOptionKind, RequestPermissionOutcome, RequestPermissionRequest,
        ToolCallLocation, ToolCallUpdate, ToolCallUpdateFields,
    };
    use tokio::sync::oneshot;

    fn options() -> Vec<PermissionOption> {
        vec![PermissionOption::new(
            "allow-once",
            "Allow once",
            PermissionOptionKind::AllowOnce,
        )]
    }

    #[test]
    fn selected_option_is_forwarded() {
        let response = permission_response(
            AcpPermissionDecision::Selected {
                option_id: "allow-once".to_string(),
            },
            &options(),
        );
        assert!(matches!(
            response.outcome,
            RequestPermissionOutcome::Selected(_)
        ));
    }

    #[test]
    fn unknown_option_is_cancelled() {
        let response = permission_response(
            AcpPermissionDecision::Selected {
                option_id: "unknown".to_string(),
            },
            &options(),
        );
        assert!(matches!(
            response.outcome,
            RequestPermissionOutcome::Cancelled
        ));
    }

    #[test]
    fn cancellation_drains_current_permission() {
        let pending = PendingPermissions::default();
        let (sender, mut receiver) = oneshot::channel();
        assert!(super::insert_permission(&pending, 1, sender).is_ok());

        super::cancel_permissions(&pending);
        assert!(matches!(
            receiver.try_recv(),
            Ok(AcpPermissionDecision::Cancelled)
        ));
    }

    #[test]
    fn cancellation_rejects_late_and_reset_allows_next_permission() {
        let pending = PendingPermissions::default();
        super::cancel_permissions(&pending);
        let (late_sender, mut late_receiver) = oneshot::channel();
        assert!(super::insert_permission(&pending, 2, late_sender).is_ok());
        assert!(matches!(
            late_receiver.try_recv(),
            Ok(AcpPermissionDecision::Cancelled)
        ));

        assert!(super::reset_permission_cancellation(&pending).is_ok());
        let (next_sender, mut next_receiver) = oneshot::channel();
        assert!(super::insert_permission(&pending, 3, next_sender).is_ok());
        assert!(
            super::resolve_permission(
                &pending,
                3,
                AcpPermissionDecision::Selected {
                    option_id: "allow-once".to_string(),
                }
            )
            .is_ok()
        );
        assert!(matches!(
            next_receiver.try_recv(),
            Ok(AcpPermissionDecision::Selected { option_id }) if option_id == "allow-once"
        ));
    }

    #[test]
    fn permission_event_preserves_the_provided_operation_context()
    -> Result<(), Box<dyn std::error::Error>> {
        let context = permission_context(permission_event(9, &operation_request()))?;
        assert_eq!(context.tool_call_id, "tool-9");
        assert_eq!(context.title.as_deref(), Some("Run provided command"));
        assert!(
            context
                .details
                .iter()
                .any(|detail| detail.contains("cat ./private.txt"))
        );
        assert!(
            context
                .details
                .iter()
                .any(|detail| detail.contains("/workspace/private.txt:9"))
        );
        Ok(())
    }

    fn operation_request() -> RequestPermissionRequest {
        let raw_input = serde_json::Value::Object(serde_json::Map::from_iter([(
            "command".to_string(),
            serde_json::Value::String("cat ./private.txt".to_string()),
        )]));
        RequestPermissionRequest::new(
            "session",
            ToolCallUpdate::new(
                "tool-9",
                ToolCallUpdateFields::new()
                    .title("Run provided command")
                    .raw_input(raw_input)
                    .locations(vec![
                        ToolCallLocation::new("/workspace/private.txt").line(9),
                    ]),
            ),
            options(),
        )
    }

    fn permission_context(
        event: crate::protocol::transport::AcpTransportEvent,
    ) -> Result<AcpPermissionContext, Box<dyn std::error::Error>> {
        match event {
            crate::protocol::transport::AcpTransportEvent::PermissionRequested {
                context, ..
            } => Ok(context),
            _ => Err("permission request must become a permission event".into()),
        }
    }
}
