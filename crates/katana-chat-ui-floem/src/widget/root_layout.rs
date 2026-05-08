use floem::prelude::*;

pub(super) struct FloemRootLayout;

impl FloemRootLayout {
    pub(super) fn full_size_layer(view: impl IntoView + 'static) -> impl IntoView {
        container(view).style(Self::full_size_style)
    }

    pub(super) fn full_size_style(style: floem::style::Style) -> floem::style::Style {
        style
            .size_full()
            .min_width(0.0)
            .min_height(0.0)
            .flex_grow(1.0)
            .flex_shrink(1.0)
    }
}
