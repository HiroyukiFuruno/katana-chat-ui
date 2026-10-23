use crate::{ChatHostError, ChatHostState};
use katana_acp_client::protocol::{
    AcpPermissionContext, AcpPermissionDecision, AcpPermissionOption,
};
use katana_chat_ui::{VendorUiOption, VendorUiPermissionRequest};
use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct PendingPermissionQueue {
    queued: VecDeque<VendorUiPermissionRequest>,
}

impl PendingPermissionQueue {
    pub(super) fn enqueue(&mut self, host: &mut ChatHostState, request: VendorUiPermissionRequest) {
        if host.session.vendor_ui_state().pending_permission.is_none() {
            show(host, request);
        } else {
            self.queued.push_back(request);
        }
    }

    pub(super) fn advance(&mut self, host: &mut ChatHostState) {
        let next = self.queued.pop_front();
        let mut vendor = host.session.vendor_ui_state().clone();
        vendor.pending_permission = next;
        host.session.set_vendor_ui_state(vendor);
    }

    pub(super) fn clear(&mut self) {
        self.queued.clear();
    }
}

pub(super) fn enqueue(
    host: &mut ChatHostState,
    queue: &mut PendingPermissionQueue,
    request_id: u64,
    context: AcpPermissionContext,
    options: Vec<AcpPermissionOption>,
) {
    queue.enqueue(host, request(request_id, context, options));
}

fn request(
    request_id: u64,
    context: AcpPermissionContext,
    options: Vec<AcpPermissionOption>,
) -> VendorUiPermissionRequest {
    VendorUiPermissionRequest {
        request_id,
        tool_call_id: Some(context.tool_call_id),
        title: context.title,
        details: context.details,
        options: options
            .into_iter()
            .map(|option| VendorUiOption {
                id: option.option_id,
                label: option.name,
            })
            .collect(),
    }
}

fn show(host: &mut ChatHostState, request: VendorUiPermissionRequest) {
    let mut vendor = host.session.vendor_ui_state().clone();
    vendor.pending_permission = Some(request);
    host.session.set_vendor_ui_state(vendor);
}

pub(super) fn clear(host: &mut ChatHostState) {
    let mut vendor = host.session.vendor_ui_state().clone();
    vendor.pending_permission = None;
    host.session.set_vendor_ui_state(vendor);
}

pub(super) fn validate(
    host: &ChatHostState,
    request_id: u64,
    decision: &AcpPermissionDecision,
) -> Result<(), ChatHostError> {
    let request = host
        .session
        .vendor_ui_state()
        .pending_permission
        .as_ref()
        .filter(|request| request.request_id == request_id)
        .ok_or_else(|| ChatHostError::Provider("Permission request is no longer pending".into()))?;
    if let AcpPermissionDecision::Selected { option_id } = decision
        && !request.options.iter().any(|option| option.id == *option_id)
    {
        return Err(ChatHostError::Provider("Unknown permission option".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use katana_acp_client::protocol::{
        AcpPermissionContext, AcpPermissionDecision, AcpPermissionOption,
    };
    use katana_chat_ui::{ChatSession, VendorUiState};

    use super::{PendingPermissionQueue, clear, enqueue, validate};
    use crate::{ChatHistoryStore, ChatHostState, ChatOutputExecutor};

    #[test]
    fn accepts_a_selected_pending_option() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());
        validate(&host, 7, &selected("allow-once"))?;
        Ok(())
    }

    #[test]
    fn projected_operation_context_reaches_the_real_kuc_frame()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());

        let request = host
            .session()
            .vendor_ui_state()
            .pending_permission
            .as_ref()
            .ok_or_else(|| std::io::Error::other("missing pending permission"))?;
        assert_eq!(request.tool_call_id.as_deref(), Some("tool-7"));
        assert_eq!(request.title.as_deref(), Some("Run command"));
        assert_eq!(request.details, ["command: cat ./README.md"]);

        let surface =
            katana_chat_ui::ChatUiSurface::from_render_model(&host.session().render_model());
        let mut runtime = katana_chat_ui::kuc::ChatUiRuntime::light(&surface)?;
        let frame = runtime.render(
            &surface,
            katana_chat_ui::kuc::ChatUiRuntimeViewport::new(640, 480, 1.0),
        )?;
        assert!(format!("{:?}", frame.tree.root()).contains("command: cat ./README.md"));
        assert!(
            frame
                .action_hits
                .iter()
                .any(|hit| hit.action.target.as_str() == "chat.permission.option.0")
        );
        Ok(())
    }

    #[test]
    fn accepts_cancellation_of_a_pending_request() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());
        validate(&host, 7, &AcpPermissionDecision::Cancelled)?;
        Ok(())
    }

    #[test]
    fn rejects_a_non_pending_request_id() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());
        assert!(validate(&host, 8, &AcpPermissionDecision::Cancelled).is_err());
        Ok(())
    }

    #[test]
    fn rejects_a_selected_option_that_was_not_offered() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());
        assert!(validate(&host, 7, &selected("unknown")).is_err());
        Ok(())
    }

    #[test]
    fn queued_requests_are_shown_in_request_id_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());
        enqueue(&mut host, &mut queue, 8, context_for("tool-8"), options());
        assert_pending(&host, 7);
        validate(&host, 7, &selected("allow-once"))?;
        queue.advance(&mut host);
        assert_pending(&host, 8);
        validate(&host, 8, &selected("allow-once"))?;
        queue.advance(&mut host);
        assert_pending(&host, 0);
        Ok(())
    }

    #[test]
    fn clearing_queue_and_ui_drops_current_and_waiting_requests()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut host = host()?;
        let mut queue = PendingPermissionQueue::default();
        enqueue(&mut host, &mut queue, 7, context(), options());
        enqueue(&mut host, &mut queue, 8, context_for("tool-8"), options());
        queue.clear();
        clear(&mut host);
        queue.advance(&mut host);
        assert_pending(&host, 0);
        Ok(())
    }

    fn assert_pending(host: &ChatHostState, request_id: u64) {
        assert_eq!(
            host.session()
                .vendor_ui_state()
                .pending_permission
                .as_ref()
                .map_or(0, |request| request.request_id),
            request_id
        );
    }

    fn host() -> Result<ChatHostState, Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(ChatHistoryStore::new_session_id());
        let mut session = ChatSession::new();
        session.set_provider_configured("KatanAgent");
        session.set_vendor_ui_state(VendorUiState::for_vendor("katanagent"));
        Ok(ChatHostState::new(
            session,
            ChatHistoryStore::new(root.clone()),
            ChatOutputExecutor::new(root),
        )?)
    }

    fn options() -> Vec<AcpPermissionOption> {
        vec![AcpPermissionOption {
            option_id: "allow-once".into(),
            name: "Allow once".into(),
        }]
    }

    fn context() -> AcpPermissionContext {
        AcpPermissionContext {
            tool_call_id: "tool-7".into(),
            title: Some("Run command".into()),
            details: vec!["command: cat ./README.md".into()],
        }
    }

    fn context_for(tool_call_id: &str) -> AcpPermissionContext {
        AcpPermissionContext {
            tool_call_id: tool_call_id.into(),
            title: Some("Read file".into()),
            details: vec!["path: ./README.md".into()],
        }
    }

    fn selected(option_id: &str) -> AcpPermissionDecision {
        AcpPermissionDecision::Selected {
            option_id: option_id.into(),
        }
    }
}
