mod history;
mod provider;
mod state;

use crossbeam_channel::Sender;
use floem::action::open_file;
use floem::file::FileDialogOptions;
use floem::prelude::*;
use floem::reactive::create_effect;
use floem::window::WindowConfig;
use katana_chat_ui::{ChatUiSurface, HostActionKind};
use katana_chat_ui_floem::{FloemChatActions, FloemChatView};
use provider::ManualProviderEvent;
use state::ManualFloemState;

const HARNESS_WINDOW_WIDTH: f64 = 1280.0;
const HARNESS_WINDOW_HEIGHT: f64 = 900.0;

struct ManualFloemHost {
    state: ManualFloemState,
}

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
        let signals = ViewSignals::new(self.state);
        let event_sender = Self::connect_provider_events(signals);
        let chat = Self::chat_view(signals, event_sender);
        Self::host_view(chat, signals)
    }

    fn connect_provider_events(signals: ViewSignals) -> Sender<ManualProviderEvent> {
        let (event_sender, event_receiver) = crossbeam_channel::unbounded::<ManualProviderEvent>();
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
        event_sender
    }

    fn chat_view(signals: ViewSignals, event_sender: Sender<ManualProviderEvent>) -> impl IntoView {
        FloemChatView::render(
            signals.surface,
            signals.draft,
            FloemChatActions {
                on_attach: Self::attach_action(signals),
                on_remove_attachment: Self::remove_attachment_action(signals),
                on_new_chat: Self::new_chat_action(signals),
                on_history: Self::history_action(signals),
                on_output_action: Self::output_action(signals),
                on_submit: Self::submit_action(signals, event_sender),
                on_stop: Self::stop_action(signals),
                on_vendor_select: Self::vendor_action(signals),
                on_control_select: Self::control_action(signals),
            },
        )
    }

    fn host_view(chat: impl IntoView + 'static, _signals: ViewSignals) -> impl IntoView {
        Self::chat_layer(chat)
    }

    fn chat_layer(chat: impl IntoView + 'static) -> impl IntoView {
        container(chat).style(|style| {
            style
                .size_full()
                .min_width(0.0)
                .min_height(0.0)
                .flex_grow(1.0)
                .flex_shrink(1.0)
        })
    }

    fn attach_action(signals: ViewSignals) -> impl Fn() + Copy + 'static {
        move || {
            signals
                .state
                .update(ManualFloemState::attach_dialog_opening);
            signals.sync();
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

    fn new_chat_action(signals: ViewSignals) -> impl Fn() + Copy + 'static {
        move || {
            signals.state.update(ManualFloemState::start_new_chat);
            signals.draft.set(String::new());
            signals.sync();
        }
    }

    fn history_action(signals: ViewSignals) -> impl Fn() + Copy + 'static {
        move || {
            signals.state.update(ManualFloemState::open_history);
            signals.sync();
        }
    }

    fn output_action(signals: ViewSignals) -> impl Fn(u64, HostActionKind) + Copy + 'static {
        move |output_id, action| {
            signals
                .state
                .update(|it| it.handle_output_action(output_id, action));
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
    floem::Application::new()
        .window(
            move |_| host.view(),
            Some(
                WindowConfig::default()
                    .title("katana-chat-ui harness floem")
                    .size((HARNESS_WINDOW_WIDTH, HARNESS_WINDOW_HEIGHT)),
            ),
        )
        .run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_window_opens_wide_enough_for_chat_ui_review() {
        assert_eq!(HARNESS_WINDOW_WIDTH, 1280.0);
        assert_eq!(HARNESS_WINDOW_HEIGHT, 900.0);
    }

    #[test]
    fn harness_mounts_chat_as_full_size_layer() {
        let source = include_str!("main.rs");

        assert!(source.contains("fn chat_layer"));
        assert!(source.contains("container(chat)"));
        assert!(source.contains(".min_height(0.0)"));
        assert!(!source.contains(&["Harness", "Panel"].concat()));
        assert!(!source.contains(&["hover", "_container"].concat()));
    }
}
