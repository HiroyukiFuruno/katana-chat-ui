use crate::{AcpProviderProjection, ChatHostError, ChatHostState};

use super::{KATANAGENT_ID, ProviderManager, ProviderSlot};

impl ProviderManager {
    pub fn select(
        &mut self,
        host: &mut ChatHostState,
        vendor_id: &str,
    ) -> Result<(), ChatHostError> {
        if selection_is_blocked(
            self.active_vendor_id.as_deref(),
            vendor_id,
            host.session().render_model().input.can_cancel,
            host.session()
                .vendor_ui_state()
                .pending_permission
                .is_some(),
        ) {
            return Err(ChatHostError::Provider(
                "Stop or respond to the active provider before selecting another provider".into(),
            ));
        }
        let models = self.inventory.ready_models(vendor_id).ok_or_else(|| {
            ChatHostError::Provider(format!("ACP provider is unavailable: {vendor_id}"))
        })?;
        if vendor_id == KATANAGENT_ID {
            self.select_katan(host, models)?;
        } else {
            self.apply_to_active(host, vendor_id, AcpProviderProjection::Ready { models })?;
        }
        self.active_vendor_id = Some(vendor_id.to_string());
        self.clear_restored_model();
        self.install_history_context(host);
        self.sync_inventory(host);
        host.persist()
    }

    pub async fn submit(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        if host.session().render_model().input.can_cancel {
            return Err(ChatHostError::Provider(
                "Stop the active provider before submitting another message".into(),
            ));
        }
        if self.restored_model_is_pending() {
            return Err(ChatHostError::Provider(
                "restored model is still reconnecting".into(),
            ));
        }
        if self.active_vendor_id.as_deref() == Some(KATANAGENT_ID) {
            self.submit_katan(host).await
        } else {
            self.active_bridge_mut()?.submit(host).await
        }
    }

    pub async fn stop(&mut self, host: &mut ChatHostState) -> Result<(), ChatHostError> {
        if self.active_vendor_id.as_deref() == Some(KATANAGENT_ID) {
            self.stop_katan(host).await
        } else {
            self.active_bridge_mut()?.stop(host).await
        }
    }

    pub async fn select_model(
        &mut self,
        host: &mut ChatHostState,
        model_id: &str,
    ) -> Result<(), ChatHostError> {
        let restored_model_pending = self.restored_model_is_pending();
        let is_katanagent = self.active_vendor_id.as_deref() == Some(KATANAGENT_ID);
        if !is_katanagent
            && self
                .active_vendor_id
                .as_deref()
                .is_some_and(|vendor_id| self.restored_model_confirmation(vendor_id).is_some())
        {
            return Err(ChatHostError::Provider(
                "restored model selection is awaiting agent confirmation".into(),
            ));
        }
        let result = if is_katanagent {
            self.katan_slot_mut()?.select_model(model_id)
        } else {
            self.active_bridge()?.select_model_id(model_id).await
        };
        if result.is_ok() {
            self.model_selection_queued(host, model_id, restored_model_pending, is_katanagent)?;
        }
        result
    }

    fn model_selection_queued(
        &mut self,
        host: &mut ChatHostState,
        model_id: &str,
        restored_model_pending: bool,
        is_katanagent: bool,
    ) -> Result<(), ChatHostError> {
        if restored_model_pending && !is_katanagent {
            let vendor_id = self
                .active_vendor_id
                .clone()
                .ok_or_else(no_active_provider)?;
            let model = self
                .inventory
                .ready_models(&vendor_id)
                .and_then(|models| models.into_iter().find(|model| model.id == model_id));
            let Some(model) = model else {
                let reason = format!("restored model selection cannot be confirmed: {model_id}");
                self.block_restored_model(reason.clone());
                host.session_mut().set_provider_missing(reason.clone());
                host.persist()?;
                return Err(ChatHostError::Provider(reason));
            };
            self.await_restored_model_confirmation(&vendor_id, model.config_id, model_id.into());
            host.session_mut().set_provider_missing(format!(
                "model selection is awaiting agent confirmation: {model_id}"
            ));
            host.persist()?;
        } else {
            self.clear_restored_model();
            if restored_model_pending {
                self.restore_connection_after_model_selection(host);
                host.persist()?;
            }
        }
        Ok(())
    }

    fn restore_connection_after_model_selection(&self, host: &mut ChatHostState) {
        if let Some(vendor_id) = self.active_vendor_id.as_deref() {
            host.session_mut().set_provider_configured(vendor_id);
        }
    }

    pub async fn respond_permission(
        &mut self,
        host: &mut ChatHostState,
        request_id: u64,
        decision: crate::AcpPermissionDecision,
    ) -> Result<(), ChatHostError> {
        self.active_bridge_mut()?
            .respond_permission(host, request_id, decision)
            .await
    }

    pub(super) async fn flush_pending_cancellations(
        &mut self,
    ) -> Result<Vec<super::ProviderPollEvent>, ChatHostError> {
        let mut pending = std::mem::take(&mut self.pending_cancellations);
        let mut events = Vec::new();
        while let Some(request) = pending.pop() {
            let result = match self.slots.get(&request.vendor_id) {
                Some(ProviderSlot::Acp(slot)) => {
                    slot.bridge.cancel_permission(request.request_id).await
                }
                Some(ProviderSlot::Katan(_)) => Err(ChatHostError::Provider(
                    "KatanAgent does not use ACP permissions".into(),
                )),
                None => Err(ChatHostError::Provider(format!(
                    "ACP slot disappeared: {}",
                    request.vendor_id
                ))),
            };
            if let Err(error) = result {
                pending.push(request);
                self.pending_cancellations = pending;
                return Err(error);
            }
            self.slots.remove(&request.vendor_id);
            events.push(super::event(
                &request.vendor_id,
                super::ProviderPollKind::CancelledInactivePermission,
            ));
        }
        Ok(events)
    }

    fn active_bridge_mut(&mut self) -> Result<&mut crate::AcpHostBridge, ChatHostError> {
        let vendor_id = self
            .active_vendor_id
            .clone()
            .ok_or_else(no_active_provider)?;
        self.slots
            .get_mut(&vendor_id)
            .and_then(|slot| match slot {
                ProviderSlot::Acp(slot) => Some(&mut slot.bridge),
                ProviderSlot::Katan(_) => None,
            })
            .ok_or_else(no_active_provider)
    }

    fn active_bridge(&self) -> Result<&crate::AcpHostBridge, ChatHostError> {
        let vendor_id = self
            .active_vendor_id
            .as_deref()
            .ok_or_else(no_active_provider)?;
        self.slots
            .get(vendor_id)
            .and_then(|slot| match slot {
                ProviderSlot::Acp(slot) => Some(&slot.bridge),
                ProviderSlot::Katan(_) => None,
            })
            .ok_or_else(no_active_provider)
    }
}

fn no_active_provider() -> ChatHostError {
    ChatHostError::Provider("No active ACP provider is ready".into())
}

fn selection_is_blocked(
    active_vendor_id: Option<&str>,
    requested_vendor_id: &str,
    is_streaming: bool,
    has_pending_permission: bool,
) -> bool {
    active_vendor_id.is_some_and(|active| active != requested_vendor_id)
        && (is_streaming || has_pending_permission)
}

#[cfg(test)]
mod tests {
    use super::super::{AcpSlot, ProviderManager, ProviderSlot, RestoredModelSelection};
    use super::selection_is_blocked;
    use crate::{
        AcpHostBridge, AcpProviderExecutor, AcpProviderModel, AcpProviderProfile, ChatHistoryStore,
        ChatHostState, ChatOutputExecutor,
    };
    use katana_chat_ui::{ChatSession, ProviderConnectionState, VendorUiState};

    #[test]
    fn active_stream_or_permission_blocks_provider_switch() {
        assert!(selection_is_blocked(
            Some("codex-cli"),
            "claude-code",
            true,
            false
        ));
        assert!(selection_is_blocked(
            Some("codex-cli"),
            "claude-code",
            false,
            true
        ));
        assert!(!selection_is_blocked(
            Some("codex-cli"),
            "codex-cli",
            true,
            true
        ));
    }

    #[test]
    fn switching_acp_provider_preserves_unreceived_history_context()
    -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let _guard = runtime.enter();
        let root = tempfile::tempdir()?;
        let mut host = history_host(root.path())?;
        let mut manager = acp_slots(root.path())?;
        manager.bind_session_identity(&mut host)?;
        assert!(pending_context(&manager, "codex-cli")?.contains("previous answer"));

        complete_active_history_for_test(&mut manager, &host)?;
        manager.select(&mut host, "claude-code")?;

        assert!(pending_context(&manager, "claude-code")?.contains("previous answer"));
        Ok(())
    }

    #[test]
    fn same_acp_provider_selection_does_not_duplicate_history_context()
    -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let _guard = runtime.enter();
        let root = tempfile::tempdir()?;
        let mut host = history_host(root.path())?;
        let mut manager = acp_slots(root.path())?;
        manager.bind_session_identity(&mut host)?;
        complete_active_history_for_test(&mut manager, &host)?;
        manager.select(&mut host, "codex-cli")?;

        assert!(pending_context(&manager, "codex-cli").is_err());
        Ok(())
    }

    #[test]
    fn switching_back_sends_only_new_history_messages() -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let _guard = runtime.enter();
        let root = tempfile::tempdir()?;
        let mut host = history_host(root.path())?;
        let mut manager = acp_slots(root.path())?;
        manager.bind_session_identity(&mut host)?;
        complete_active_history_for_test(&mut manager, &host)?;
        manager.select(&mut host, "claude-code")?;
        append_turn(&mut host, "new question", "new answer")?;
        complete_active_history_for_test(&mut manager, &host)?;
        manager.select(&mut host, "codex-cli")?;

        let context = pending_context(&manager, "codex-cli")?;
        assert!(context.contains("new answer"));
        assert!(!context.contains("previous answer"));
        Ok(())
    }

    fn complete_active_history_for_test(
        manager: &mut ProviderManager,
        host: &ChatHostState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(vendor_id) = manager.active_vendor_id.clone() else {
            return Err("missing active provider".into());
        };
        let Some(ProviderSlot::Acp(slot)) = manager.slots.get_mut(&vendor_id) else {
            return Err("missing active ACP slot".into());
        };
        slot.bridge.complete_history_context_for_test(host);
        Ok(())
    }

    fn history_host(root: &std::path::Path) -> Result<ChatHostState, Box<dyn std::error::Error>> {
        let mut host = host(root, "codex-cli")?;
        host.session_mut().set_provider_configured("codex-cli");
        append_turn(&mut host, "previous question", "previous answer")?;
        Ok(host)
    }

    fn append_turn(
        host: &mut ChatHostState,
        question: &str,
        answer: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        host.session_mut().draft_mut().set_text(question);
        host.session_mut().submit_draft()?;
        host.session_mut().start_assistant_stream(answer)?;
        host.session_mut().finish_assistant_message()?;
        Ok(())
    }

    fn acp_slots(root: &std::path::Path) -> Result<ProviderManager, Box<dyn std::error::Error>> {
        let mut manager = ProviderManager::new();
        for vendor_id in ["codex-cli", "claude-code"] {
            let profile = AcpProviderProfile::new(vendor_id, "/bin/cat", root);
            let bridge = AcpHostBridge::new(AcpProviderExecutor::connect(profile)?);
            manager.slots.insert(
                vendor_id.into(),
                ProviderSlot::Acp(Box::new(AcpSlot { bridge })),
            );
            manager
                .inventory
                .ready(vendor_id, vec![acp_model(vendor_id)]);
        }
        Ok(manager)
    }

    fn acp_model(vendor_id: &str) -> AcpProviderModel {
        AcpProviderModel {
            config_id: format!("{vendor_id}-config"),
            id: format!("{vendor_id}-model"),
            name: format!("{vendor_id} model"),
            selected: true,
        }
    }

    fn pending_context(
        manager: &ProviderManager,
        vendor_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let Some(ProviderSlot::Acp(slot)) = manager.slots.get(vendor_id) else {
            return Err(format!("missing ACP slot: {vendor_id}").into());
        };
        slot.bridge
            .pending_history_text_for_test()
            .ok_or_else(|| format!("missing history context: {vendor_id}").into())
    }

    #[test]
    fn failed_explicit_model_selection_keeps_a_blocked_connection_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut host = host(root.path(), "katanagent")?;
        let mut manager = ProviderManager::new();
        manager.active_vendor_id = Some("katanagent".into());
        manager.restored_model = Some(RestoredModelSelection::Blocked {
            vendor_id: "katanagent".into(),
            reason: "restored model is unavailable: saved".into(),
        });

        host.session_mut()
            .set_provider_missing("restored model is unavailable: saved");

        let runtime = tokio::runtime::Builder::new_current_thread().build()?;
        assert!(
            runtime
                .block_on(manager.select_model(&mut host, "replacement"))
                .is_err()
        );
        assert!(manager.restored_model_is_pending());
        assert!(!host.session().render_model().provider.is_configured());
        Ok(())
    }

    #[test]
    fn queued_restored_acp_selection_waits_for_agent_confirmation()
    -> Result<(), Box<dyn std::error::Error>> {
        let (mut manager, mut host, _root) = acp_restore_selection_fixture()?;
        manager.model_selection_queued(&mut host, "replacement", true, false)?;
        assert_acp_restore_waits(&manager, &host);
        Ok(())
    }

    fn acp_restore_selection_fixture()
    -> Result<(ProviderManager, ChatHostState, tempfile::TempDir), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let mut host = host(root.path(), "codex-cli")?;
        host.session_mut().set_vendor_ui_state(
            VendorUiState::for_vendor("codex-cli")
                .with_models(vec!["saved".into(), "replacement".into()], "saved"),
        );
        host.session_mut().set_provider_missing("restoring model");
        let mut manager = ProviderManager::new();
        manager.active_vendor_id = Some("codex-cli".into());
        manager.restored_model = Some(RestoredModelSelection::Applying {
            vendor_id: "codex-cli".into(),
            model_id: "saved".into(),
            config_id: "model".into(),
        });
        manager.inventory.ready(
            "codex-cli",
            vec![AcpProviderModel {
                config_id: "model".into(),
                id: "replacement".into(),
                name: "Replacement".into(),
                selected: false,
            }],
        );
        Ok((manager, host, root))
    }

    fn assert_acp_restore_waits(manager: &ProviderManager, host: &ChatHostState) {
        assert!(matches!(
            &manager.restored_model,
            Some(RestoredModelSelection::AwaitingConfirmation {
                model_id,
                config_id,
                ..
            }) if model_id == "replacement" && config_id == "model"
        ));
        assert_eq!(
            host.session().vendor_ui_state().selected_model.as_deref(),
            Some("saved")
        );
        assert!(matches!(
            host.session().render_model().provider,
            ProviderConnectionState::Missing(_)
        ));
    }

    #[test]
    fn katanagent_restored_selection_remains_synchronous() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = tempfile::tempdir()?;
        let mut host = host(root.path(), "katanagent")?;
        host.session_mut().set_provider_missing("restoring model");
        let mut manager = ProviderManager::new();
        manager.active_vendor_id = Some("katanagent".into());
        manager.restored_model = Some(RestoredModelSelection::Applying {
            vendor_id: "katanagent".into(),
            model_id: "saved".into(),
            config_id: "model".into(),
        });

        manager.model_selection_queued(&mut host, "replacement", true, true)?;

        assert_eq!(manager.restored_model, None);
        assert_eq!(
            host.session().render_model().provider,
            ProviderConnectionState::Configured("katanagent".into())
        );
        Ok(())
    }

    fn host(
        root: &std::path::Path,
        vendor_id: &str,
    ) -> Result<ChatHostState, Box<dyn std::error::Error>> {
        let mut session = ChatSession::new();
        session.set_vendor_ui_state(VendorUiState::for_vendor(vendor_id));
        ChatHostState::new(
            session,
            ChatHistoryStore::new(root.into()),
            ChatOutputExecutor::new(root.into()),
        )
        .map_err(Into::into)
    }
}
