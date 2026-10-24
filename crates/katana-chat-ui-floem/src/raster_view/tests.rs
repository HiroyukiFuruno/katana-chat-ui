use floem::{
    event::Event,
    kurbo::{Point, Vec2},
    pointer::{PointerButton, PointerInputEvent, PointerWheelEvent},
    reactive::RwSignal,
    style::Style,
    taffy::{
        TaffyTree,
        prelude::{AvailableSpace, Size},
    },
};
use katana_chat_ui::{
    ChatSession, ChatUiSurface,
    kuc::{ChatUiRuntime, ChatUiRuntimeEffect, ChatUiRuntimeInput, ChatUiRuntimeViewport},
};

use super::{
    FloemChatView, FloemKucCallbacks, events::event_input, image::image, raster_view_style,
};

mod ime;

#[test]
fn standard_mount_builds_a_kuc_runtime_view() -> Result<(), String> {
    let surface = RwSignal::new(ready_surface());
    let draft = RwSignal::new("draft".to_string());
    let view = FloemChatView::mount(
        surface,
        draft,
        1.0,
        FloemKucCallbacks {
            on_action: |_| {},
            on_draft: |_| {},
        },
    );

    view.map(|_| ()).map_err(|error| error.to_string())
}

#[test]
fn mounted_view_style_fills_a_nonzero_floem_layout() -> Result<(), String> {
    let mut layout = TaffyTree::<()>::new();
    let child = layout
        .new_leaf(raster_view_style(Style::new()).to_taffy_style())
        .map_err(|error| error.to_string())?;
    let root = layout
        .new_with_children(Style::new().size(640.0, 480.0).to_taffy_style(), &[child])
        .map_err(|error| error.to_string())?;
    let available = Size {
        width: AvailableSpace::Definite(640.0),
        height: AvailableSpace::Definite(480.0),
    };

    layout
        .compute_layout(root, available)
        .map_err(|error| error.to_string())?;
    let size = layout
        .layout(child)
        .map_err(|error| error.to_string())?
        .size;
    assert_eq!((size.width, size.height), (640.0, 480.0));
    Ok(())
}

#[test]
fn raster_image_is_opaque_rgba_in_source_rgb_order() {
    let (image, hash) = image(&katana_chat_ui::kuc::ChatUiRuntimeRgba {
        width: 2,
        height: 1,
        pixels: vec![0x0012_3456, 0x00AB_CDEF],
    });

    assert_eq!(image.width, 2);
    assert_eq!(image.height, 1);
    assert_eq!(image.alpha, u8::MAX);
    assert_eq!(
        image.data.as_ref(),
        &[0x12, 0x34, 0x56, 255, 0xAB, 0xCD, 0xEF, 255]
    );
    assert_eq!(hash.len(), 8);
}

#[test]
fn native_pointer_and_ime_events_reach_kuc_effects() -> Result<(), String> {
    let surface = ready_surface();
    let mut runtime = ChatUiRuntime::light(&surface).map_err(|error| error.to_string())?;
    focus_with_native_pointer(&mut runtime, &surface)?;
    commit_with_native_ime(&mut runtime, &surface)?;

    assert!(matches!(
        event_input(&wheel(), false),
        Some(ChatUiRuntimeInput::Scroll { delta_y: 12, .. })
    ));
    assert!(matches!(
        event_input(&Event::FocusLost, false),
        Some(ChatUiRuntimeInput::FocusLost)
    ));
    Ok(())
}

fn focus_with_native_pointer(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
) -> Result<(), String> {
    let frame = runtime
        .render(surface, viewport())
        .map_err(|error| error.to_string())?;
    let input = event_input(&pointer_down(&frame), false).ok_or("pointer was not mapped")?;
    let effects = runtime
        .dispatch(&frame, input)
        .map_err(|error| error.to_string())?;
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, ChatUiRuntimeEffect::FocusComposer))
    );
    Ok(())
}

fn commit_with_native_ime(
    runtime: &mut ChatUiRuntime,
    surface: &ChatUiSurface,
) -> Result<(), String> {
    let frame = runtime
        .render(surface, viewport())
        .map_err(|error| error.to_string())?;
    let input =
        event_input(&Event::ImeCommit("日本".to_string()), false).ok_or("IME was not mapped")?;
    let effects = runtime
        .dispatch(&frame, input)
        .map_err(|error| error.to_string())?;
    assert!(
        effects.iter().any(
            |effect| matches!(effect, ChatUiRuntimeEffect::DraftChanged(text) if text == "日本")
        )
    );
    Ok(())
}

fn ready_surface() -> ChatUiSurface {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    ChatUiSurface::from_render_model(&session.render_model())
}

fn viewport() -> ChatUiRuntimeViewport {
    ChatUiRuntimeViewport::new(640, 480, 1.0)
}

fn pointer_down(frame: &katana_chat_ui::kuc::ChatUiRuntimeFrame) -> Event {
    let bounds = frame.composer.bounds;
    Event::PointerDown(PointerInputEvent {
        pos: Point::new(f64::from(bounds.x), f64::from(bounds.y)),
        button: PointerButton::Primary,
        modifiers: Default::default(),
        count: 1,
    })
}

fn wheel() -> Event {
    Event::PointerWheel(PointerWheelEvent {
        pos: Point::ZERO,
        delta: Vec2::new(0.0, 12.0),
        modifiers: Default::default(),
    })
}
