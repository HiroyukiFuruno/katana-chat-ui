use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};

use tokio::{
    runtime::Builder,
    sync::{mpsc, watch},
};

use crate::ChatHostState;

use super::{ManagedHostUpdate, SequencedCommand, run};

pub(in crate::managed_host) fn spawn(
    root: PathBuf,
    host: ChatHostState,
    commands: mpsc::Receiver<SequencedCommand>,
    updates: watch::Sender<ManagedHostUpdate>,
    events: mpsc::UnboundedSender<ManagedHostUpdate>,
    shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    thread::spawn(
        move || match Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime.block_on(run(root, host, commands, updates, events, shutdown)),
            Err(error) => report_runtime_error(&events, &host, error),
        },
    )
}

fn report_runtime_error(
    events: &mpsc::UnboundedSender<ManagedHostUpdate>,
    host: &ChatHostState,
    error: std::io::Error,
) {
    let _ = events.send(ManagedHostUpdate {
        ack_seq: 0,
        surface: host.surface(),
        effect: None,
        error: Some(format!(
            "managed host runtime initialization failed: {error}"
        )),
    });
}
