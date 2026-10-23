use super::{AcpProviderProjection, content, project, project_update};
use agent_client_protocol::schema::v1::{
    ContentBlock, ImageContent, Plan, SessionUpdate, ToolCall, ToolCallUpdate,
    ToolCallUpdateFields, ToolKind,
};
use katana_acp_client::protocol::{AcpPermissionContext, AcpPermissionOption, AcpTransportEvent};
use katana_chat_ui::AgentActivityKind;

#[test]
fn config_acknowledgement_does_not_start_assistant_activity() {
    assert_eq!(
        project(AcpTransportEvent::ConfigUpdated {
            config_id: "model".into()
        }),
        AcpProviderProjection::ConfigUpdated {
            config_id: "model".into()
        },
    );
}

#[test]
fn rejected_config_projects_as_a_nonfatal_configuration_error() {
    assert_eq!(
        project(AcpTransportEvent::ConfigRejected {
            config_id: "model".into(),
            message: "Invalid params".into(),
        }),
        AcpProviderProjection::ConfigRejected {
            config_id: "model".into(),
            message: "Invalid params".into(),
        },
    );
}

#[test]
fn permission_projection_preserves_request_and_display_labels() {
    let options = vec![AcpPermissionOption {
        option_id: "allow_once".into(),
        name: "今回だけ許可".into(),
    }];
    assert_eq!(
        project(AcpTransportEvent::PermissionRequested {
            request_id: 42,
            session_id: "session".into(),
            context: AcpPermissionContext {
                tool_call_id: "tool-42".into(),
                title: Some("Run the provided command".into()),
                details: vec!["command: rm -i ./review-target".into()],
            },
            options: options.clone(),
        }),
        AcpProviderProjection::Permission {
            request_id: 42,
            context: AcpPermissionContext {
                tool_call_id: "tool-42".into(),
                title: Some("Run the provided command".into()),
                details: vec!["command: rm -i ./review-target".into()],
            },
            options
        },
    );
}

#[test]
fn tool_calls_preserve_their_typed_activity() {
    for (kind, expected) in tool_kind_projections() {
        assert_eq!(project_update(tool_call(kind)), activity(expected));
    }
}

#[test]
fn tool_kind_update_changes_the_current_activity() {
    assert_eq!(
        project_update(tool_call_update(Some(ToolKind::Fetch))),
        activity(AgentActivityKind::WebSearching),
    );
}

#[test]
fn tool_kindless_update_leaves_the_current_activity_unchanged() {
    assert_eq!(
        project_update(tool_call_update(None)),
        AcpProviderProjection::Unchanged,
    );
}

#[test]
fn plan_and_nontext_updates_keep_processing_activity() {
    assert_eq!(
        project_update(SessionUpdate::Plan(Plan::new(Vec::new()))),
        activity(AgentActivityKind::Processing),
    );
    let image = ContentBlock::Image(ImageContent::new("data", "image/png"));
    assert_eq!(
        content(image, AcpProviderProjection::Text),
        activity(AgentActivityKind::Processing),
    );
}

fn tool_kind_projections() -> [(ToolKind, AgentActivityKind); 9] {
    [
        (ToolKind::Read, AgentActivityKind::Reading),
        (ToolKind::Edit, AgentActivityKind::Editing),
        (ToolKind::Delete, AgentActivityKind::Editing),
        (ToolKind::Move, AgentActivityKind::Editing),
        (ToolKind::Search, AgentActivityKind::Searching),
        (ToolKind::Fetch, AgentActivityKind::WebSearching),
        (ToolKind::Execute, AgentActivityKind::Executing),
        (ToolKind::Think, AgentActivityKind::Processing),
        (ToolKind::Other, AgentActivityKind::Processing),
    ]
}

fn tool_call(kind: ToolKind) -> SessionUpdate {
    SessionUpdate::ToolCall(ToolCall::new("tool", "tool activity").kind(kind))
}

fn tool_call_update(kind: Option<ToolKind>) -> SessionUpdate {
    let fields = match kind {
        Some(kind) => ToolCallUpdateFields::new().kind(kind),
        None => ToolCallUpdateFields::new(),
    };
    SessionUpdate::ToolCallUpdate(ToolCallUpdate::new("tool", fields))
}

fn activity(kind: AgentActivityKind) -> AcpProviderProjection {
    AcpProviderProjection::Activity(kind)
}
