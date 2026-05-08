use super::{FloemComposerControlsView, styles};
mod actions;
mod attachments;
mod editor;
mod state;

use floem::{AnyView, peniko::Color, prelude::*, views::Stack};
use katana_chat_ui::{
    ChatUiComposerSurface, ChatUiSlashLauncherSurface, ChatUiSurface, ChatUiUsageSurface,
    ChatUiVendorBarSurface,
};

const COMPOSER_PADDING: f64 = 14.0;
const SLASH_LAUNCHER_GAP: f64 = 6.0;
const SLASH_LAUNCHER_PADDING: f64 = 10.0;

pub use actions::FloemComposerActions;
pub(super) use state::{ComposerDraftSubmitter, ComposerEditorState, ComposerSubmitGate};

pub struct FloemComposerView;

impl FloemComposerView {
    pub fn composer<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>(
        surface: RwSignal<ChatUiSurface>,
        draft: RwSignal<String>,
        actions: FloemComposerActions<
            OnAttach,
            OnRemoveAttachment,
            OnSubmit,
            OnStop,
            OnControlSelect,
        >,
    ) -> impl IntoView
    where
        OnAttach: Fn() + Copy + 'static,
        OnRemoveAttachment: Fn(usize) + Copy + 'static,
        OnSubmit: Fn(String) + Clone + 'static,
        OnStop: Fn() + Copy + 'static,
        OnControlSelect: Fn(String, String) + Copy + 'static,
    {
        composer_body(surface, draft, actions)
    }
}

#[derive(Clone)]
struct ComposerSurface {
    composer: ChatUiComposerSurface,
    usage: ChatUiUsageSurface,
    vendor: ChatUiVendorBarSurface,
}

impl ComposerSurface {
    fn from_surface(surface: ChatUiSurface) -> Self {
        Self {
            composer: surface.composer,
            usage: surface.usage,
            vendor: surface.vendor_bar,
        }
    }
}

fn composer_body<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>(
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    actions: FloemComposerActions<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>,
) -> impl IntoView
where
    OnAttach: Fn() + Copy + 'static,
    OnRemoveAttachment: Fn(usize) + Copy + 'static,
    OnSubmit: Fn(String) + Clone + 'static,
    OnStop: Fn() + Copy + 'static,
    OnControlSelect: Fn(String, String) + Copy + 'static,
{
    let composer = surface.get_untracked().composer;
    initialize_draft(&composer, draft);
    let editor_text = ComposerEditorState::initial_text(&composer, draft);
    let editor = editor::ComposerEditorView::render(
        surface,
        composer.placeholder,
        editor_text,
        draft,
        actions.on_submit.clone(),
    );
    composer_frame(v_stack((
        editor,
        slash_launcher(surface),
        attachments::ComposerAttachmentTray::render(surface, actions.on_remove_attachment),
        controls(surface, draft, actions),
    )))
}

fn slash_launcher(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
    dyn_container(
        move || surface.get().composer.slash_launcher,
        |launcher| {
            if !launcher.visible {
                return empty().into_any();
            }
            slash_launcher_panel(launcher)
        },
    )
}

fn slash_launcher_panel(launcher: ChatUiSlashLauncherSurface) -> AnyView {
    dyn_stack(
        move || launcher.entries.clone(),
        |entry| entry.id.clone(),
        |entry| text(format!("/{}  {}", entry.id, entry.label)),
    )
    .style(|style| {
        style
            .gap(SLASH_LAUNCHER_GAP)
            .padding(SLASH_LAUNCHER_PADDING)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::TOOLTIP_RADIUS)
            .background(Color::WHITE)
            .color(styles::COLOR_TEXT)
            .font_size(styles::FONT_META)
            .flex_col()
    })
    .into_any()
}

fn composer_frame(frame: Stack) -> impl IntoView {
    h_stack((frame.style(|style| {
        style
            .width_full()
            .max_width(styles::CHAT_BODY_MAX_WIDTH)
            .min_width(0.0)
            .flex_shrink(0.0)
            .gap(styles::PANEL_GAP)
            .flex_col()
            .padding(COMPOSER_PADDING)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::BUBBLE_RADIUS)
            .background(Color::WHITE)
    }),))
    .style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .items_center()
            .justify_center()
    })
}

fn controls<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>(
    surface: RwSignal<ChatUiSurface>,
    draft: RwSignal<String>,
    actions: FloemComposerActions<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>,
) -> impl IntoView
where
    OnAttach: Fn() + Copy + 'static,
    OnRemoveAttachment: Fn(usize) + Copy + 'static,
    OnSubmit: Fn(String) + Clone + 'static,
    OnStop: Fn() + Copy + 'static,
    OnControlSelect: Fn(String, String) + Copy + 'static,
{
    dyn_container(
        move || ComposerSurface::from_surface(surface.get()),
        move |surface| {
            FloemComposerControlsView::render(
                surface.composer,
                surface.usage,
                surface.vendor,
                draft,
                actions.control_actions(),
            )
        },
    )
}

fn initialize_draft(composer: &ChatUiComposerSurface, draft: RwSignal<String>) {
    let should_sync = draft.get_untracked().is_empty() && !composer.text.is_empty();
    if should_sync {
        draft.set(composer.text.clone());
    }
}
