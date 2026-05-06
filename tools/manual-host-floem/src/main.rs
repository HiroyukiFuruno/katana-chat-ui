mod harness_panel;
mod ollama;
mod provider;
mod state;

use crossbeam_channel::Sender;
use floem::action::open_file;
use floem::file::FileDialogOptions;
use floem::prelude::*;
use floem::reactive::create_effect;
use harness_panel::HarnessPanel;
use katana_chat_ui::ChatUiSurface;
use katana_chat_ui_floem::{FloemChatActions, FloemChatView};
use provider::ManualProviderEvent;
use state::ManualFloemState;

struct ManualFloemHost {
    state: ManualFloemState,
}

const HOST_PADDING: f64 = 24.0;

#[derive(Clone, Copy)]
pub(crate) struct ViewSignals {
    pub(crate) state: RwSignal<ManualFloemState>,
    surface: RwSignal<ChatUiSurface>,
    pub(crate) last_event: RwSignal<String>,
    pub(crate) vendor_summary: RwSignal<String>,
    draft: RwSignal<String>,
}

impl ViewSignals {
    fn new(state: ManualFloemState) -> Self {
        let state = RwSignal::new(state);
        Self {
            state,
            surface: RwSignal::new(state.get().surface()),
            last_event: RwSignal::new(state.get().last_event),
            vendor_summary: RwSignal::new(state.get().vendor_summary()),
            draft: RwSignal::new(String::new()),
        }
    }

    pub(crate) fn sync(self) {
        let snapshot = self.state.get();
        self.surface.set(snapshot.surface());
        self.vendor_summary.set(snapshot.vendor_summary());
        self.last_event.set(snapshot.last_event);
    }
}

impl ManualFloemHost {
    fn new() -> Result<Self, state::ManualFloemError> {
        Ok(Self {
            state: ManualFloemState::new()?,
        })
    }

    fn view(self) -> impl IntoView {
        let (event_sender, event_receiver) =
            crossbeam_channel::unbounded::<ManualProviderEvent>();
        let signals = ViewSignals::new(self.state);
        let event_signal = floem::ext_event::create_signal_from_channel(event_receiver);
        create_effect(move |_| {
            let Some(event) = event_signal.get() else {
                return;
            };
            signals
                .state
                .update(move |it| it.apply_provider_event(event));
            signals.sync();
        });
        let chat = FloemChatView::render(
            signals.surface,
            signals.draft,
            FloemChatActions::new(
                Self::attach_action(signals),
                Self::remove_attachment_action(signals),
                Self::settings_action(signals),
                Self::submit_action(signals, event_sender),
                Self::stop_action(signals),
                Self::vendor_action(signals),
                Self::control_action(signals),
            ),
        );
        container(stack((chat, HarnessPanel::render(signals))))
        .style(|style| {
            style
                .size_full()
                .min_width(0.0)
                .height_full()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .padding(HOST_PADDING)
        })
    }

    fn attach_action(signals: ViewSignals) -> impl Fn() + Copy + 'static {
        move || {
            open_file(
                FileDialogOptions::new()
                    .title("Attach files")
                    .multi_selection(),
                move |file_info| {
                    match file_info {
                        Some(file_info) => signals
                            .state
                            .update(move |it| it.attach_selected_paths(file_info.path)),
                        None => signals.state.update(ManualFloemState::attach_cancelled),
                    }
                    signals.sync();
                },
            );
        }
    }

    fn remove_attachment_action(signals: ViewSignals) -> impl Fn(usize) + Copy + 'static {
        move |index| {
            signals.state.update(|it| it.remove_attachment(index));
            signals.sync();
        }
    }

    fn settings_action(signals: ViewSignals) -> impl Fn() + Copy + 'static {
        move || {
            signals
                .state
                .update(ManualFloemState::refresh_ollama_models);
            signals.sync();
        }
    }

    fn submit_action(
        signals: ViewSignals,
        event_sender: Sender<ManualProviderEvent>,
    ) -> impl Fn(String) + Clone + 'static {
        move |text| {
            let job = signals
                .state
                .try_update(|it| it.start_submit(text))
                .flatten();
            if job.is_some() {
                signals.draft.set(String::new());
            }
            signals.sync();
            if let Some(job) = job {
                let sender = event_sender.clone();
                std::thread::spawn(move || {
                    provider::ManualProviderExecutor::execute(job, sender);
                });
            }
        }
    }

    fn stop_action(signals: ViewSignals) -> impl Fn() + Copy + 'static {
        move || {
            signals.state.update(ManualFloemState::stop);
            signals.sync();
        }
    }

    fn vendor_action(signals: ViewSignals) -> impl Fn(String) + Copy + 'static {
        move |vendor_id| {
            signals.state.update(|it| it.select_vendor(vendor_id));
            signals.sync();
        }
    }

    fn control_action(signals: ViewSignals) -> impl Fn(String, String) + Copy + 'static {
        move |key, value| {
            signals.state.update(|it| it.select_control(key, value));
            signals.sync();
        }
    }
}

fn main() {
    let host = match ManualFloemHost::new() {
        Ok(host) => host,
        Err(error) => {
            eprintln!("manual-host-floem failed to start: {error}");
            return;
        }
    };
    floem::launch(move || host.view());
}
