use std::{path::PathBuf, time::Duration};

use tokio::{
    sync::{mpsc, watch},
    time,
};

use crate::{ChatHostEffect, ChatHostState, ProviderManager};

use super::{ManagedHostCommand, ManagedHostUpdate};

mod dispatch;
mod runtime;

use dispatch::dispatch;
pub(super) use runtime::spawn;

const POLL_INTERVAL: Duration = Duration::from_millis(50);
pub(super) struct SequencedCommand {
    pub(super) seq: u64,
    pub(super) command: ManagedHostCommand,
}
async fn run(
    root: PathBuf,
    mut host: ChatHostState,
    mut commands: mpsc::Receiver<SequencedCommand>,
    updates: watch::Sender<ManagedHostUpdate>,
    events: mpsc::UnboundedSender<ManagedHostUpdate>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut providers = initialize_providers(root, &mut host, &updates, &events);
    let mut ticker = time::interval(POLL_INTERVAL);
    let mut acknowledged_seq = 0;
    ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    while !*shutdown.borrow() {
        tokio::select! {
            command = commands.recv() => {
                let Some(command) = command else { break; };
                let Some(seq) = execute(command, &mut host, &mut providers, &updates, &events, &mut shutdown).await else { break; };
                acknowledged_seq = seq;
            },
            _ = ticker.tick() => {
                if !poll_until_shutdown(&mut host, &mut providers, &updates, &events, acknowledged_seq, &mut shutdown).await { break; }
            },
            _ = shutdown.changed() => break,
        }
    }
}

fn initialize_providers(
    root: PathBuf,
    host: &mut ChatHostState,
    updates: &watch::Sender<ManagedHostUpdate>,
    events: &mpsc::UnboundedSender<ManagedHostUpdate>,
) -> ProviderManager {
    let mut providers = ProviderManager::new();
    providers.probe(root);
    let startup_error = providers.bind_session_identity(host).err();
    publish_snapshot(updates, 0, host);
    if let Some(error) = startup_error {
        publish_event(events, 0, host, None, Some(error.to_string()));
    }
    providers
}

async fn poll_until_shutdown(
    host: &mut ChatHostState,
    providers: &mut ProviderManager,
    updates: &watch::Sender<ManagedHostUpdate>,
    events: &mpsc::UnboundedSender<ManagedHostUpdate>,
    acknowledged_seq: u64,
    shutdown: &mut watch::Receiver<bool>,
) -> bool {
    tokio::select! {
        _ = poll(host, providers, updates, events, acknowledged_seq) => true,
        _ = shutdown.changed() => false,
    }
}

async fn execute(
    command: SequencedCommand,
    host: &mut ChatHostState,
    providers: &mut ProviderManager,
    updates: &watch::Sender<ManagedHostUpdate>,
    events: &mpsc::UnboundedSender<ManagedHostUpdate>,
    shutdown: &mut watch::Receiver<bool>,
) -> Option<u64> {
    let seq = command.seq;
    let result = tokio::select! {
        result = dispatch(command.command, host, providers) => result,
        _ = shutdown.changed() => return None,
    };
    match result {
        Ok(Some(effect)) => publish_event(events, seq, host, Some(effect), None),
        Ok(None) => {}
        Err(error) => publish_event(events, seq, host, None, Some(error.to_string())),
    }
    publish_snapshot(updates, seq, host);
    Some(seq)
}

async fn poll(
    host: &mut ChatHostState,
    providers: &mut ProviderManager,
    updates: &watch::Sender<ManagedHostUpdate>,
    event_updates: &mpsc::UnboundedSender<ManagedHostUpdate>,
    acknowledged_seq: u64,
) {
    match providers.poll(host).await {
        Ok(events) if !events.is_empty() => publish_snapshot(updates, acknowledged_seq, host),
        Ok(_) => {}
        Err(error) => publish_event(
            event_updates,
            acknowledged_seq,
            host,
            None,
            Some(error.to_string()),
        ),
    }
}

fn publish_snapshot(
    updates: &watch::Sender<ManagedHostUpdate>,
    ack_seq: u64,
    host: &ChatHostState,
) {
    updates.send_replace(update(ack_seq, host, None, None));
}

fn publish_event(
    updates: &mpsc::UnboundedSender<ManagedHostUpdate>,
    ack_seq: u64,
    host: &ChatHostState,
    effect: Option<ChatHostEffect>,
    error: Option<String>,
) {
    let _ = updates.send(update(ack_seq, host, effect, error));
}

fn update(
    ack_seq: u64,
    host: &ChatHostState,
    effect: Option<ChatHostEffect>,
    error: Option<String>,
) -> ManagedHostUpdate {
    ManagedHostUpdate {
        ack_seq,
        surface: host.surface(),
        effect,
        error,
    }
}
