mod fixture;
mod state;
#[path = "../../manual-host-trace/mod.rs"]
mod trace;

use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::AppContext;
use gpui::{
    App, Application, Bounds, Context, Entity, IntoElement, ParentElement, Render, Styled,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, size,
};
use katana_chat_ui::{
    ChatUiSurface,
    kuc::{ChatUiRuntime, ChatUiRuntimeEffect},
};
use katana_chat_ui_gpui::GpuiKucView;

use self::state::ManualGpuiState;

const HARNESS_WINDOW_WIDTH: f32 = 1280.0;
const HARNESS_WINDOW_HEIGHT: f32 = 900.0;

struct ManualGpuiHost {
    view: Entity<GpuiKucView>,
    state: Rc<RefCell<ManualGpuiState>>,
}

struct PreparedManualGpuiHost {
    runtime: ChatUiRuntime,
    state: Rc<RefCell<ManualGpuiState>>,
    surface: ChatUiSurface,
}

impl PreparedManualGpuiHost {
    fn new() -> Result<Self, String> {
        let state = Rc::new(RefCell::new(ManualGpuiState::new()?));
        let surface = state.borrow().surface();
        let runtime = GpuiKucView::prepare(&surface).map_err(|error| error.to_string())?;
        Ok(Self {
            runtime,
            state,
            surface,
        })
    }

    fn mount(self, cx: &mut Context<ManualGpuiHost>) -> ManualGpuiHost {
        let Self {
            runtime,
            state,
            surface,
        } = self;
        let effect_state = state.clone();
        let view = cx.new(move |view_cx| {
            GpuiKucView::from_runtime(runtime, surface, effect_handler(effect_state), view_cx)
        });
        start_provider_poll(cx);
        ManualGpuiHost { view, state }
    }
}

fn start_provider_poll(cx: &mut Context<ManualGpuiHost>) {
    cx.spawn(async move |host, cx| {
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            if host.update(cx, |host, cx| host.poll_provider(cx)).is_err() {
                break;
            }
        }
    })
    .detach();
}

impl ManualGpuiHost {
    fn poll_provider(&mut self, cx: &mut Context<Self>) {
        let mut state = self.state.borrow_mut();
        match state.poll() {
            Ok(true) => self
                .view
                .update(cx, |view, cx| view.set_surface(state.surface(), cx)),
            Ok(false) => {}
            Err(error) => eprintln!("manual-host-gpui provider failed: {error}"),
        }
        if let Some(error) = state.take_error() {
            eprintln!("manual-host-gpui action failed: {error}");
        }
    }
}

fn effect_handler(
    state: Rc<RefCell<ManualGpuiState>>,
) -> impl FnMut(ChatUiRuntimeEffect, &mut Context<GpuiKucView>) -> Option<ChatUiSurface> {
    move |effect, _| {
        let mut state = state.borrow_mut();
        state.apply_runtime_effect(effect);
        if let Some(error) = state.take_error() {
            eprintln!("manual-host-gpui action failed: {error}");
        }
        Some(state.surface())
    }
}

impl Render for ManualGpuiHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.view.clone())
    }
}

fn main() {
    let fixture = fixture_or_exit();
    Application::new().run(|cx: &mut App| match fixture {
        Some(surface) => launch_fixture(surface, cx),
        None => launch_live(cx),
    });
}

fn fixture_or_exit() -> Option<fixture::Fixture> {
    match fixture::load_from_args() {
        Ok(fixture) => fixture,
        Err(error) => fatal_fixture_startup(&error),
    }
}

fn launch_fixture(fixture: fixture::Fixture, cx: &mut App) {
    if let Err(error) = fixture::open(fixture, cx) {
        fatal_fixture(&error);
    }
    cx.activate(true);
}

fn launch_live(cx: &mut App) {
    let prepared = match PreparedManualGpuiHost::new() {
        Ok(host) => host,
        Err(error) => {
            eprintln!("manual-host-gpui failed to build model: {error}");
            return;
        }
    };
    let bounds = Bounds::centered(
        None,
        size(px(HARNESS_WINDOW_WIDTH), px(HARNESS_WINDOW_HEIGHT)),
        cx,
    );
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("katana-chat-ui harness gpui".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    if let Err(error) = cx.open_window(options, move |_, cx| cx.new(|cx| prepared.mount(cx))) {
        eprintln!("manual-host-gpui failed to start: {error}");
        return;
    }
    cx.activate(true);
}

fn fatal_fixture_startup(error: &str) -> ! {
    eprintln!("manual-host-gpui fixture failed to start: {error}");
    std::process::exit(1)
}

fn fatal_fixture(error: &str) -> ! {
    eprintln!("manual-host-gpui fixture rendering failed: {error}");
    std::process::exit(1)
}

#[cfg(test)]
mod tests {
    use super::{HARNESS_WINDOW_HEIGHT, HARNESS_WINDOW_WIDTH};

    #[test]
    fn harness_window_size_matches_floem_baseline() {
        assert_eq!(HARNESS_WINDOW_WIDTH, 1280.0);
        assert_eq!(HARNESS_WINDOW_HEIGHT, 900.0);
    }
}
