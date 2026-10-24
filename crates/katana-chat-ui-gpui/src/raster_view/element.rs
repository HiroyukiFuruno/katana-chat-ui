use std::sync::Arc;

use gpui::{
    App, Bounds, Corners, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, RenderImage, Style, Window, relative,
};

use super::GpuiKucView;

pub(super) struct KucRasterElement {
    view: Entity<GpuiKucView>,
}

impl KucRasterElement {
    pub(super) fn new(view: Entity<GpuiKucView>) -> Self {
        Self { view }
    }
}

impl IntoElement for KucRasterElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for KucRasterElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<Arc<RenderImage>>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        match self.view.update(cx, |view, _| view.redraw(bounds, window)) {
            Ok(()) => self.view.read(cx).image.clone(),
            Err(error) => {
                eprintln!("GPUI KUC raster redraw failed: {error}");
                None
            }
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        image: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.view.read(cx).focus_handle.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );
        if let Some(image) = image.take()
            && let Err(error) = window.paint_image(bounds, Corners::default(), image, 0, false)
        {
            eprintln!("GPUI KUC raster image paint failed: {error}");
        }
    }
}
