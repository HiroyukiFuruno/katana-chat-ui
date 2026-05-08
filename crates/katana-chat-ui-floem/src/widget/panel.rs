use super::styles;
use floem::{AnyView, prelude::*};

pub trait FloemPanelSlotView {
    fn into_slot_view(self) -> AnyView;
}

impl<View> FloemPanelSlotView for View
where
    View: IntoView + 'static,
{
    fn into_slot_view(self) -> AnyView {
        self.into_any()
    }
}

pub struct FloemPanelSlot {
    view: AnyView,
}

impl FloemPanelSlot {
    pub fn new(view: impl FloemPanelSlotView) -> Self {
        Self {
            view: view.into_slot_view(),
        }
    }
}

#[derive(Default)]
pub struct FloemChatPanel {
    header: Option<FloemPanelSlot>,
    thread: Option<FloemPanelSlot>,
    composer: Option<FloemPanelSlot>,
    extensions: Vec<FloemPanelSlot>,
}

impl FloemChatPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn header(mut self, slot: FloemPanelSlot) -> Self {
        self.header = Some(slot);
        self
    }

    pub fn thread(mut self, slot: FloemPanelSlot) -> Self {
        self.thread = Some(slot);
        self
    }

    pub fn extension(mut self, slot: FloemPanelSlot) -> Self {
        self.extensions.push(slot);
        self
    }

    pub fn composer(mut self, slot: FloemPanelSlot) -> Self {
        self.composer = Some(slot);
        self
    }

    pub fn render(self) -> impl IntoView {
        container(
            v_stack((
                slot_view(self.header),
                slot_view(self.thread),
                v_stack_from_iter(self.extensions.into_iter().map(|slot| slot.view)),
                slot_view(self.composer),
            ))
            .style(styles::FloemWidgetStyle::root),
        )
        .style(|style| {
            style
                .size_full()
                .min_width(0.0)
                .min_height(0.0)
                .flex_grow(1.0)
                .flex_shrink(1.0)
        })
    }
}

fn slot_view(slot: Option<FloemPanelSlot>) -> AnyView {
    match slot {
        Some(slot) => slot.view,
        None => empty().into_any(),
    }
}
