mod fixture;
mod managed_state;
#[path = "../../manual-host-trace/mod.rs"]
mod trace;

#[cfg(test)]
mod history;
#[cfg(test)]
mod provider;
#[cfg(test)]
mod state;

use floem::views::{Decorators, container, label};
use floem::window::WindowConfig;
use floem::{AnyView, Application, IntoView, View};
use katana_chat_ui::kuc::ChatUiAction;
use katana_chat_ui_floem::{FloemChatView, FloemKucCallbacks};
use kcu_host_support::ChatHostEffect;
use managed_state::ManagedFloemState;

const HARNESS_WINDOW_WIDTH: f64 = 1280.0;
const HARNESS_WINDOW_HEIGHT: f64 = 900.0;

struct ManualFloemHost {
    state: ManagedFloemState,
}

impl ManualFloemHost {
    fn new() -> Result<Self, String> {
        let root = std::env::current_dir().map_err(|error| error.to_string())?;
        Ok(Self {
            state: ManagedFloemState::spawn(root)?,
        })
    }

    fn view(self, renderer_control: bool) -> AnyView {
        self.state.clone().schedule_updates(Self::handle_effect);
        match FloemChatView::mount(
            self.state.surface,
            self.state.draft,
            1.0,
            FloemKucCallbacks {
                on_action: Self::action_handler(self.state.clone()),
                on_draft: Self::draft_handler(self.state.clone()),
            },
        ) {
            Ok(chat) => {
                let chat = if renderer_control {
                    let chat_id = chat.id();
                    chat.on_resize(move |_| {
                        let rect = chat_id.get_content_rect();
                        eprintln!(
                            "FLOEM RENDERER CONTROL RASTER_RECT width={:.0} height={:.0}",
                            rect.width(),
                            rect.height()
                        );
                    })
                    .into_any()
                } else {
                    chat.into_any()
                };
                Self::chat_container(chat).into_any()
            }
            Err(error) => Self::mount_failure(error.to_string()).into_any(),
        }
    }

    fn chat_container(view: impl IntoView + 'static) -> impl IntoView {
        container(view).style(Self::chat_style)
    }

    fn chat_style(style: floem::style::Style) -> floem::style::Style {
        style
            .size_full()
            .min_width(0.0)
            .min_height(0.0)
            .flex_grow(1.0_f32)
            .flex_shrink(1.0_f32)
    }

    fn mount_failure(error: String) -> impl IntoView {
        eprintln!("manual-host-floem KUC mount failed: {error}");
        let message = format!("KUC mount failed: {error}");
        container(label(move || message.clone())).style(Self::chat_style)
    }

    fn action_handler(state: ManagedFloemState) -> impl Fn(ChatUiAction) + 'static {
        move |action| state.send_action(action)
    }

    fn draft_handler(state: ManagedFloemState) -> impl Fn(String) + 'static {
        move |text| state.send_draft(text)
    }

    fn handle_effect(effect: ChatHostEffect) {
        match effect {
            ChatHostEffect::RequestAttachment => {
                eprintln!("manual-host-floem attachment picker is not available");
            }
            ChatHostEffect::ProviderSelectionRequired { vendor_id } => {
                eprintln!("manual-host-floem provider selection required: {vendor_id}");
            }
            ChatHostEffect::None
            | ChatHostEffect::Submitted { .. }
            | ChatHostEffect::OutputApplied { .. }
            | ChatHostEffect::OutputReverted { .. } => {
                // KUC surface update already contains the visible state for these effects.
            }
        }
    }
}

fn main() {
    let renderer_control = match fixture::renderer_control_enabled() {
        Ok(enabled) => enabled,
        Err(error) => fatal_fixture_startup(&error),
    };
    let fixture = match fixture::load_from_args() {
        Ok(fixture) => fixture,
        Err(error) => fatal_fixture_startup(&error),
    };
    if let Some(fixture) = fixture {
        run_fixture(fixture, renderer_control);
        return;
    }
    let host = match ManualFloemHost::new() {
        Ok(host) => host,
        Err(error) => {
            eprintln!("manual-host-floem failed to start: {error}");
            return;
        }
    };
    run_host(
        move || controlled_view(host.view(renderer_control), renderer_control),
        (HARNESS_WINDOW_WIDTH, HARNESS_WINDOW_HEIGHT),
    );
}

fn run_fixture(fixture: fixture::Fixture, renderer_control: bool) {
    let size = (f64::from(fixture.width), f64::from(fixture.height));
    match fixture::view(fixture.surface) {
        Ok(view) => run_host(move || controlled_view(view, renderer_control), size),
        Err(error) => fatal_fixture(&error),
    }
}

fn controlled_view(view: AnyView, renderer_control: bool) -> AnyView {
    if !renderer_control {
        return view;
    }
    let view_id = view.id();
    let view = view.on_resize(move |_| {
        let rect = view_id.get_content_rect();
        eprintln!(
            "FLOEM RENDERER CONTROL CONTAINER_RECT width={:.0} height={:.0}",
            rect.width(),
            rect.height()
        );
    });
    fixture::with_renderer_control(view)
}

fn fatal_fixture_startup(error: &str) -> ! {
    eprintln!("manual-host-floem fixture failed to start: {error}");
    std::process::exit(1)
}

fn fatal_fixture(error: &str) -> ! {
    eprintln!("manual-host-floem fixture rendering failed: {error}");
    std::process::exit(1)
}

fn run_host(view: impl FnOnce() -> AnyView + 'static, size: (f64, f64)) {
    Application::new()
        .window(
            move |_| view(),
            Some(
                WindowConfig::default()
                    .title("katana-chat-ui harness floem")
                    .size(size),
            ),
        )
        .run();
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_WINDOW_HEIGHT, HARNESS_WINDOW_WIDTH};

    #[test]
    fn harness_window_opens_wide_enough_for_chat_ui_review() {
        assert_eq!(HARNESS_WINDOW_WIDTH, 1280.0);
        assert_eq!(HARNESS_WINDOW_HEIGHT, 900.0);
    }
}
