use crate::{
    AcpProviderExecutor, AcpProviderModel, AcpProviderProjection, ChatHostError, ChatHostState,
};
use katana_acp_client::protocol::AcpPermissionDecision;
use katana_chat_ui::AgentActivityKind;

mod activity;
mod history_context;
pub(crate) mod lifecycle;
mod permission;
mod poll;

use permission::PendingPermissionQueue;

pub struct AcpHostBridge {
    executor: AcpProviderExecutor,
    streaming: bool,
    cancelling: bool,
    pending_history_context: Option<katana_acp_client::AcpContentBlock>,
    pending_permissions: PendingPermissionQueue,
}

impl AcpHostBridge {
    pub fn new(executor: AcpProviderExecutor) -> Self {
        Self {
            executor,
            streaming: false,
            cancelling: false,
            pending_history_context: None,
            pending_permissions: PendingPermissionQueue::default(),
        }
    }

    pub fn is_available(&self) -> bool {
        self.executor.is_available()
    }

    pub async fn submit(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        self.ensure_not_cancelling()?;
        let mut blocks = self
            .pending_history_context
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        blocks.extend(host.session.draft().to_acp_content_blocks());
        host.submit()?;
        host.session.start_assistant_stream(String::new())?;
        self.streaming = true;
        match self.executor.prompt(blocks).await.map_err(provider_error) {
            Ok(()) => {
                self.pending_history_context = None;
                host.persist()
            }
            Err(error) => self.prompt_failed(host, error),
        }
    }

    pub async fn stop(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        if !self.streaming || self.cancelling {
            return Ok(());
        }
        self.executor.cancel().await.map_err(provider_error)?;
        self.cancelling = true;
        self.pending_permissions.clear();
        permission::clear(host);
        host.persist()?;
        Ok(())
    }

    pub async fn select_model(&self, model: &AcpProviderModel) -> Result<(), ChatHostError> {
        self.ensure_not_cancelling()?;
        self.executor
            .select_model(model)
            .await
            .map_err(provider_error)
    }

    pub async fn select_model_id(&self, id: &str) -> Result<(), ChatHostError> {
        self.ensure_not_cancelling()?;
        self.executor
            .select_model_id(id)
            .await
            .map_err(provider_error)
    }

    pub async fn respond_permission(
        &mut self,
        host: &mut ChatHostState,
        request_id: u64,
        decision: AcpPermissionDecision,
    ) -> Result<(), ChatHostError> {
        self.ensure_not_cancelling()?;
        permission::validate(host, request_id, &decision)?;
        self.executor
            .choose_permission(request_id, decision)
            .await
            .map_err(provider_error)?;
        self.pending_permissions.advance(host);
        host.persist()?;
        Ok(())
    }

    pub async fn cancel_permission(&self, request_id: u64) -> Result<(), ChatHostError> {
        self.executor
            .choose_permission(request_id, AcpPermissionDecision::Cancelled)
            .await
            .map_err(provider_error)
    }

    pub fn try_next_projection(&mut self) -> Option<AcpProviderProjection> {
        self.executor.try_next_projection()
    }

    pub fn apply_projection(
        &mut self,
        host: &mut ChatHostState,
        projection: AcpProviderProjection,
    ) -> Result<(), ChatHostError> {
        if self.cancelling {
            return self.apply_cancelling_projection(host, projection);
        }
        self.apply_active_projection(host, projection)
    }

    fn apply_active_projection(
        &mut self,
        host: &mut ChatHostState,
        projection: AcpProviderProjection,
    ) -> Result<(), ChatHostError> {
        match projection {
            AcpProviderProjection::Connecting => self.connecting(host),
            AcpProviderProjection::Unchanged | AcpProviderProjection::ConfigUpdated { .. } => {
                Ok(())
            }
            AcpProviderProjection::Ready { models } => self.ready(host, models),
            AcpProviderProjection::Permission {
                request_id,
                context,
                options,
            } => self.queue_permission(host, request_id, context, options),
            AcpProviderProjection::ConfigRejected { message, .. } => {
                lifecycle::config_rejected(host, message)
            }
            AcpProviderProjection::Unavailable(reason) | AcpProviderProjection::Failed(reason) => {
                self.fail(host, reason)
            }
            stream => self.apply_stream(host, stream),
        }
    }

    fn connecting(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        self.pending_permissions.clear();
        lifecycle::connecting(host);
        Ok(())
    }

    fn ready(
        &mut self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        lifecycle::ready(host, &self.executor.profile.vendor_id, &models, false);
        Ok(())
    }

    fn queue_permission(
        &mut self,
        host: &mut ChatHostState,
        request_id: u64,
        context: katana_acp_client::protocol::AcpPermissionContext,
        options: Vec<katana_acp_client::protocol::AcpPermissionOption>,
    ) -> Result<(), ChatHostError> {
        permission::enqueue(
            host,
            &mut self.pending_permissions,
            request_id,
            context,
            options,
        );
        Ok(())
    }

    pub(crate) fn apply_restored_ready_projection(
        &mut self,
        host: &mut ChatHostState,
        models: Vec<AcpProviderModel>,
    ) -> Result<(), ChatHostError> {
        lifecycle::ready(host, &self.executor.profile.vendor_id, &models, true);
        Ok(())
    }

    fn apply_stream(
        &mut self,
        host: &mut ChatHostState,
        projection: AcpProviderProjection,
    ) -> Result<(), ChatHostError> {
        match projection {
            AcpProviderProjection::Text(text) => self.text(host, text),
            AcpProviderProjection::Thinking(text) => self.thinking(host, text),
            AcpProviderProjection::Activity(kind) => self.activity(host, kind),
            AcpProviderProjection::Finished => self.finish(host),
            _ => Ok(()),
        }
    }

    fn apply_cancelling_projection(
        &mut self,
        host: &mut ChatHostState,
        projection: AcpProviderProjection,
    ) -> Result<(), ChatHostError> {
        match projection {
            AcpProviderProjection::Text(_)
            | AcpProviderProjection::Thinking(_)
            | AcpProviderProjection::Activity(_)
            | AcpProviderProjection::Permission { .. } => Ok(()),
            AcpProviderProjection::Finished => {
                self.cancelling = false;
                self.finish(host)
            }
            AcpProviderProjection::Unavailable(reason) | AcpProviderProjection::Failed(reason) => {
                self.cancelling = false;
                self.fail(host, reason)
            }
            // 設定の確定・拒否は取消対象の応答本文とは独立して反映する。
            configuration => self.apply_active_projection(host, configuration),
        }
    }

    fn text(&mut self, host: &mut ChatHostState, text: String) -> Result<(), ChatHostError> {
        self.ensure_stream(host)?;
        host.session.append_assistant_chunk(text)?;
        host.session
            .set_assistant_activity(AgentActivityKind::Generating)?;
        Ok(())
    }

    fn thinking(&mut self, host: &mut ChatHostState, text: String) -> Result<(), ChatHostError> {
        self.ensure_stream(host)?;
        host.session.append_assistant_thinking("Thinking", text)?;
        Ok(())
    }

    fn activity(
        &mut self,
        host: &mut ChatHostState,
        kind: AgentActivityKind,
    ) -> Result<(), ChatHostError> {
        self.ensure_stream(host)?;
        activity::project(host, kind)
    }

    fn finish(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        self.pending_permissions.clear();
        lifecycle::complete(host, &mut self.streaming)
    }

    fn fail(&mut self, host: &mut ChatHostState, reason: String) -> Result<(), ChatHostError> {
        self.pending_permissions.clear();
        lifecycle::fail(host, &mut self.streaming, reason)
    }

    fn prompt_failed(
        &mut self,
        host: &mut ChatHostState,
        error: ChatHostError,
    ) -> Result<(), ChatHostError> {
        self.cancelling = false;
        self.pending_permissions.clear();
        lifecycle::fail(host, &mut self.streaming, error.to_string())?;
        host.persist()?;
        Err(error)
    }

    fn ensure_stream(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        if !self.streaming {
            host.session.start_assistant_stream(String::new())?;
            self.streaming = true;
        }
        Ok(())
    }

    fn ensure_not_cancelling(&self) -> Result<(), ChatHostError> {
        if self.cancelling {
            return Err(ChatHostError::Provider(
                "ACP provider is still cancelling the active prompt".into(),
            ));
        }
        Ok(())
    }
}

fn provider_error(error: katana_acp_client::AcpError) -> ChatHostError {
    ChatHostError::Provider(error.to_string())
}

#[cfg(test)]
mod tests;
