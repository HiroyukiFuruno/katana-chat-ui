use super::*;

const COMMAND_BUFFER: usize = 4;
const EVENT_BUFFER: usize = 64;

pub(super) struct WireSetup {
    pub(super) inputs: WireTaskInputs,
    pub(super) commands:
        tokio::sync::mpsc::Sender<crate::protocol::transport::types::AcpTransportCommand>,
    pub(super) received: tokio::sync::mpsc::Receiver<AcpTransportEvent>,
    pub(super) first_rx: tokio::sync::mpsc::Receiver<()>,
    pub(super) second_rx: tokio::sync::mpsc::Receiver<()>,
    pub(super) cancel_rx: tokio::sync::mpsc::Receiver<()>,
    pub(super) release: Arc<tokio::sync::Notify>,
    pub(super) done: tokio::sync::oneshot::Sender<()>,
    pub(super) seen: Arc<AtomicUsize>,
}

pub(super) fn prepare_wire_setup(
    configs: super::super::super::super::config_state::SessionConfigState,
) -> WireSetup {
    let (client_transport, server_transport) = Channel::duplex();
    let (commands, command_rx) = tokio::sync::mpsc::channel(COMMAND_BUFFER);
    let (events, received) = tokio::sync::mpsc::channel(EVENT_BUFFER);
    let (state, first_rx, second_rx) = wire_observer();
    let seen = Arc::clone(&state.seen);
    let release = Arc::clone(&state.release_first);
    let (cancel, cancel_rx) = tokio::sync::mpsc::channel(1);
    let (done, done_rx) = tokio::sync::oneshot::channel();
    WireSetup {
        inputs: WireTaskInputs {
            client_transport,
            server_transport,
            command_rx,
            configs,
            events,
            state,
            cancel,
            done: done_rx,
        },
        commands,
        received,
        first_rx,
        second_rx,
        cancel_rx,
        release,
        done,
        seen,
    }
}

fn wire_observer() -> (
    WireServerState,
    tokio::sync::mpsc::Receiver<()>,
    tokio::sync::mpsc::Receiver<()>,
) {
    let (first, first_rx) = tokio::sync::mpsc::channel(1);
    let (second, second_rx) = tokio::sync::mpsc::channel(1);
    let state = WireServerState {
        seen: Arc::new(AtomicUsize::new(0)),
        first_received: Arc::new(first),
        second_received: Arc::new(second),
        release_first: Arc::new(tokio::sync::Notify::new()),
    };
    (state, first_rx, second_rx)
}

pub(super) fn spawn_wire_tasks(inputs: WireTaskInputs) -> WireTaskHandles {
    let WireTaskInputs {
        client_transport,
        server_transport,
        command_rx,
        configs,
        events,
        state,
        cancel,
        done,
    } = inputs;
    WireTaskHandles {
        client: tokio::spawn(run_wire_client(
            client_transport,
            command_rx,
            configs,
            events,
        )),
        server: tokio::spawn(run_wire_server(server_transport, state, cancel, done)),
    }
}
