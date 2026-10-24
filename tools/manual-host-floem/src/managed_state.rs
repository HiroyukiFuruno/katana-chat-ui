mod synchronization;

use std::{
    path::PathBuf,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use floem::{
    action::exec_after,
    reactive::{RwSignal, SignalUpdate},
};
use katana_chat_ui::{ChatUiAction, ChatUiSurface};
use kcu_host_support::{ChatHostEffect, ManagedChatHost, ManagedHostCommand, ManagedHostUpdate};
use synchronization::{drain_bounded, update_plan, upgrade_host};

use crate::trace::TraceWriter;

const UPDATE_INTERVAL: Duration = Duration::from_millis(16);
const MAX_UPDATES_PER_TICK: usize = 32;

#[derive(Clone)]
pub(crate) struct ManagedFloemState {
    host: Arc<ManagedChatHost>,
    pub(crate) surface: RwSignal<ChatUiSurface>,
    pub(crate) draft: RwSignal<String>,
    pub(crate) last_error: RwSignal<Option<String>>,
    pub(crate) last_effect: RwSignal<Option<ChatHostEffect>>,
    latest_draft_seq: Arc<AtomicU64>,
    trace: Option<TraceWriter>,
}

#[derive(Clone)]
struct UpdateObserver {
    trace: Option<TraceWriter>,
    on_effect: Arc<dyn Fn(ChatHostEffect)>,
}

impl UpdateObserver {
    fn update(&self, update: &ManagedHostUpdate) {
        if let Some(trace) = &self.trace {
            trace.update("floem", update);
        }
    }

    fn effect(&self, effect: ChatHostEffect) {
        (self.on_effect)(effect);
    }
}

impl ManagedFloemState {
    pub(crate) fn spawn(root: PathBuf) -> Result<Self, String> {
        let trace = TraceWriter::from_environment()?;
        let host = Arc::new(ManagedChatHost::spawn(root).map_err(|error| error.to_string())?);
        let surface = host.initial_surface();
        Ok(Self {
            host,
            draft: RwSignal::new(surface.composer.text.clone()),
            surface: RwSignal::new(surface),
            last_error: RwSignal::new(None),
            last_effect: RwSignal::new(None),
            latest_draft_seq: Arc::new(AtomicU64::new(0)),
            trace,
        })
    }

    pub(crate) fn send_action(&self, action: ChatUiAction) {
        let _ = self.send(ManagedHostCommand::Action(action));
    }

    pub(crate) fn send_draft(&self, text: String) {
        self.draft.set(text.clone());
        if let Some(seq) = self.send(ManagedHostCommand::DraftChanged(text)) {
            self.latest_draft_seq.store(seq, Ordering::Release);
        }
    }

    pub(crate) fn schedule_updates(&self, on_effect: impl Fn(ChatHostEffect) + 'static) {
        schedule_updates(
            Arc::downgrade(&self.host),
            self.surface,
            self.draft,
            self.last_error,
            self.last_effect,
            self.latest_draft_seq.clone(),
            UpdateObserver {
                trace: self.trace.clone(),
                on_effect: Arc::new(on_effect),
            },
        );
    }

    fn send(&self, command: ManagedHostCommand) -> Option<u64> {
        match self.host.send(command.clone()) {
            Ok(seq) => {
                if let Some(trace) = &self.trace {
                    trace.command("floem", seq, &command);
                }
                Some(seq)
            }
            Err(error) => {
                self.last_error.set(Some(error.to_string()));
                None
            }
        }
    }
}

fn schedule_updates(
    host: Weak<ManagedChatHost>,
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    error: RwSignal<Option<String>>,
    effect: RwSignal<Option<ChatHostEffect>>,
    latest: Arc<AtomicU64>,
    observer: UpdateObserver,
) {
    exec_after(UPDATE_INTERVAL, move |_| {
        schedule_tick(host, surface, draft, error, effect, latest, observer)
    });
}

fn schedule_tick(
    host: Weak<ManagedChatHost>,
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    error: RwSignal<Option<String>>,
    effect: RwSignal<Option<ChatHostEffect>>,
    latest: Arc<AtomicU64>,
    observer: UpdateObserver,
) {
    let Some(host) = upgrade_host(&host) else {
        return;
    };
    let next_observer = observer.clone();
    drain_bounded(
        || host.try_recv(),
        |update| {
            apply_effect(
                surface,
                draft,
                error,
                effect,
                &latest,
                &next_observer,
                update,
            )
        },
        |value| error.set(Some(value.to_string())),
    );
    schedule_updates(
        Arc::downgrade(&host),
        surface,
        draft,
        error,
        effect,
        latest,
        observer,
    );
}

fn apply_effect(
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    error: RwSignal<Option<String>>,
    effect: RwSignal<Option<ChatHostEffect>>,
    latest: &AtomicU64,
    observer: &UpdateObserver,
    update: ManagedHostUpdate,
) {
    if let Some(effect_value) =
        apply_update(surface, draft, error, effect, latest, observer, update)
    {
        observer.effect(effect_value);
    }
}

fn apply_update(
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    error: RwSignal<Option<String>>,
    effect: RwSignal<Option<ChatHostEffect>>,
    latest: &AtomicU64,
    observer: &UpdateObserver,
    update: ManagedHostUpdate,
) -> Option<ChatHostEffect> {
    observer.update(&update);
    let plan = update_plan(latest.load(Ordering::Acquire), &update);
    if let Some(text) = plan.draft {
        draft.set(text);
    }
    surface.set(update.surface);
    if let Some(value) = plan.error {
        error.set(Some(value));
    }
    if let Some(value) = plan.effect.as_ref() {
        effect.set(Some(value.clone()));
    }
    plan.effect
}

#[cfg(test)]
mod tests;
