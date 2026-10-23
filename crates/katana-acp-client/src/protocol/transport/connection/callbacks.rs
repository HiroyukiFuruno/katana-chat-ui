use super::super::{
    config_state::SessionConfigState,
    session_config::project_config_options,
    state::{
        PendingPermissions, PermissionRegistration, insert_permission, permission_event,
        permission_response,
    },
    types::{AcpPermissionDecision, AcpTransportEvent},
};
use agent_client_protocol::schema::v1::{
    RequestPermissionRequest, RequestPermissionResponse, SessionNotification, SessionUpdate,
};
use agent_client_protocol::{Error, Responder};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub(super) struct ConnectionCallbacks {
    events: mpsc::Sender<AcpTransportEvent>,
    pending: PendingPermissions,
    configs: SessionConfigState,
    next_permission_id: Arc<AtomicU64>,
}

impl ConnectionCallbacks {
    pub(super) fn new(
        events: mpsc::Sender<AcpTransportEvent>,
        pending: PendingPermissions,
        configs: SessionConfigState,
    ) -> Self {
        Self {
            events,
            pending,
            configs,
            next_permission_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub(super) async fn notification(
        &self,
        notification: SessionNotification,
    ) -> Result<(), Error> {
        self.send_config_options_update(&notification).await?;
        let _ = self
            .events
            .send(AcpTransportEvent::SessionUpdate {
                session_id: notification.session_id.to_string(),
                update: Box::new(notification.update),
            })
            .await;
        Ok(())
    }

    pub(super) async fn permission(
        &self,
        request: RequestPermissionRequest,
        responder: Responder<RequestPermissionResponse>,
    ) -> Result<(), Error> {
        let request_id = self.next_permission_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = oneshot::channel();
        if matches!(
            insert_permission(&self.pending, request_id, sender)?,
            PermissionRegistration::Cancelled
        ) {
            return responder.respond(permission_response(
                AcpPermissionDecision::Cancelled,
                &request.options,
            ));
        }
        let _ = self
            .events
            .send(permission_event(request_id, &request))
            .await;
        let decision = match receiver.await {
            Ok(decision) => decision,
            Err(_) => AcpPermissionDecision::Cancelled,
        };
        responder.respond(permission_response(decision, &request.options))
    }

    async fn send_config_options_update(
        &self,
        notification: &SessionNotification,
    ) -> Result<(), Error> {
        let SessionUpdate::ConfigOptionUpdate(update) = &notification.update else {
            return Ok(());
        };
        self.configs
            .publish(
                AcpTransportEvent::SessionConfigOptionsUpdated {
                    session_id: notification.session_id.to_string(),
                    config_options: project_config_options(update.config_options.clone()),
                },
                &self.events,
            )
            .await
    }
}

#[cfg(test)]
mod tests;
