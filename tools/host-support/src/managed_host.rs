//! UI スレッドから切り離した chat host の送受信境界。

mod worker;

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    thread::JoinHandle,
};

use katana_chat_ui::{Attachment, ChatSession, ChatUiAction, ChatUiSurface};
use thiserror::Error;
use tokio::sync::{mpsc, watch};

use crate::{
    ChatHistoryStore, ChatHostEffect, ChatHostError, ChatHostState, ChatOutputExecutor,
    ChatProviderPoll,
};

const COMMAND_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedHostCommand {
    Action(ChatUiAction),
    DraftChanged(String),
    AddAttachment(Attachment),
}

#[derive(Debug, Clone)]
pub struct ManagedHostUpdate {
    pub ack_seq: u64,
    pub surface: ChatUiSurface,
    pub effect: Option<ChatHostEffect>,
    pub error: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManagedHostTransportError {
    #[error("managed host command queue is full")]
    Full,
    #[error("managed host worker has stopped")]
    Stopped,
    #[error("managed host update queue is unavailable")]
    UpdatesUnavailable,
    #[error("managed host sequence is exhausted")]
    SequenceExhausted,
}

pub struct ManagedChatHost {
    commands: Mutex<CommandTransport>,
    updates: Mutex<watch::Receiver<ManagedHostUpdate>>,
    events: Mutex<mpsc::UnboundedReceiver<ManagedHostUpdate>>,
    shutdown: watch::Sender<bool>,
    worker: Option<JoinHandle<()>>,
    initial_surface: ChatUiSurface,
}

struct CommandTransport {
    sender: mpsc::Sender<worker::SequencedCommand>,
    next_seq: u64,
}

impl ManagedChatHost {
    pub fn spawn(root: impl Into<PathBuf>) -> Result<Self, ChatHostError> {
        let root = absolute_root(root.into())?;
        let (host, initial_surface) = initial_host(&root)?;
        let (commands, command_receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (update_sender, updates) = watch::channel(ManagedHostUpdate {
            ack_seq: 0,
            surface: initial_surface.clone(),
            effect: None,
            error: None,
        });
        let (shutdown, shutdown_receiver) = watch::channel(false);
        let (event_sender, events) = mpsc::unbounded_channel();
        let worker = worker::spawn(
            root,
            host,
            command_receiver,
            update_sender,
            event_sender,
            shutdown_receiver,
        );
        Ok(Self {
            commands: Mutex::new(CommandTransport {
                sender: commands,
                next_seq: 1,
            }),
            updates: Mutex::new(updates),
            events: Mutex::new(events),
            shutdown,
            worker: Some(worker),
            initial_surface,
        })
    }

    pub fn initial_surface(&self) -> ChatUiSurface {
        self.initial_surface.clone()
    }

    pub fn send(&self, command: ManagedHostCommand) -> Result<u64, ManagedHostTransportError> {
        let mut commands = self
            .commands
            .lock()
            .map_err(|_| ManagedHostTransportError::Stopped)?;
        let seq = commands.next_seq;
        let next_seq = seq
            .checked_add(1)
            .ok_or(ManagedHostTransportError::SequenceExhausted)?;
        commands
            .sender
            .try_send(worker::SequencedCommand { seq, command })
            .map_err(command_error)?;
        commands.next_seq = next_seq;
        Ok(seq)
    }

    pub fn try_recv(&self) -> Result<Option<ManagedHostUpdate>, ManagedHostTransportError> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| ManagedHostTransportError::UpdatesUnavailable)?;
        match events.try_recv() {
            Ok(update) => return Ok(Some(update)),
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {}
        }
        let mut updates = self
            .updates
            .lock()
            .map_err(|_| ManagedHostTransportError::UpdatesUnavailable)?;
        match updates.has_changed() {
            Ok(false) => Ok(None),
            Ok(true) => Ok(Some(updates.borrow_and_update().clone())),
            Err(_) => Err(ManagedHostTransportError::Stopped),
        }
    }
}

fn initial_host(root: &Path) -> Result<(ChatHostState, ChatUiSurface), ChatHostError> {
    let mut host = ChatHostState::new(
        ChatSession::new(),
        ChatHistoryStore::new(root.to_path_buf()),
        ChatOutputExecutor::new(root.to_path_buf()),
    )?;
    host.apply_provider_poll(unavailable_poll(&host));
    let surface = host.surface();
    Ok((host, surface))
}

fn absolute_root(root: PathBuf) -> Result<PathBuf, ChatHostError> {
    if root.is_absolute() {
        return Ok(root);
    }
    std::env::current_dir()
        .map(|current| current.join(root))
        .map_err(|error| ChatHostError::History(error.to_string()))
}

fn unavailable_poll(host: &ChatHostState) -> ChatProviderPoll {
    let mut state = host.session().vendor_ui_state().clone();
    state.available_vendor_ids.clear();
    ChatProviderPoll {
        state,
        unavailable_reason: Some("ACP provider probe is pending".into()),
    }
}

impl Drop for ManagedChatHost {
    fn drop(&mut self) {
        self.shutdown.send_replace(true);
        self.worker.take();
    }
}

fn command_error(
    error: mpsc::error::TrySendError<worker::SequencedCommand>,
) -> ManagedHostTransportError {
    match error {
        mpsc::error::TrySendError::Full(_) => ManagedHostTransportError::Full,
        mpsc::error::TrySendError::Closed(_) => ManagedHostTransportError::Stopped,
    }
}
