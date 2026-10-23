use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use floem::peniko::{Blob, Format, Image};
use katana_chat_ui::kuc::ChatUiRuntimeRgba;

pub(super) fn image(rgba: &ChatUiRuntimeRgba) -> (Image, Vec<u8>) {
    let bytes: Vec<u8> = rgba
        .pixels
        .iter()
        .flat_map(|pixel| {
            [
                (pixel >> 16) as u8,
                (pixel >> 8) as u8,
                *pixel as u8,
                u8::MAX,
            ]
        })
        .collect();
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let hash = hasher.finish().to_le_bytes().to_vec();
    (
        Image::new(
            Blob::new(Arc::new(bytes)),
            Format::Rgba8,
            rgba.width,
            rgba.height,
        ),
        hash,
    )
}
