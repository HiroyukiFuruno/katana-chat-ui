use super::types::AcpTransportEvent;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

#[derive(Clone, Default)]
pub(crate) struct SessionConfigState {
    ids: Arc<Mutex<BTreeMap<String, BTreeSet<String>>>>,
}

impl SessionConfigState {
    pub(crate) fn contains(&self, session_id: &str, config_id: &str) -> Result<bool, String> {
        let ids = self
            .ids
            .lock()
            .map_err(|_| "session configuration state poisoned".to_string())?;
        Ok(ids
            .get(session_id)
            .is_some_and(|ids| ids.contains(config_id)))
    }

    pub(crate) async fn publish(
        &self,
        event: AcpTransportEvent,
        events: &mpsc::Sender<AcpTransportEvent>,
    ) -> Result<(), agent_client_protocol::Error> {
        let (session_id, options) = match &event {
            AcpTransportEvent::SessionStarted {
                session_id,
                config_options,
                ..
            }
            | AcpTransportEvent::SessionConfigOptionsUpdated {
                session_id,
                config_options,
            } => (session_id, config_options),
            _ => {
                return Err(agent_client_protocol::Error::internal_error()
                    .data("expected a session configuration event"));
            }
        };
        let ids = options.iter().map(|option| option.id.clone()).collect();
        let permit = events.reserve().await.map_err(|error| {
            agent_client_protocol::Error::internal_error().data(error.to_string())
        })?;
        let mut state = self.ids.lock().map_err(|_| {
            agent_client_protocol::Error::internal_error()
                .data("session configuration state poisoned")
        })?;
        // 候補の受理状態とUI通知の順序を同じロック内で確定し、awaitをまたがない。
        state.insert(session_id.clone(), ids);
        permit.send(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
