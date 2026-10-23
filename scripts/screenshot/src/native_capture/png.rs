use anyhow::{Context, Result};
use std::path::Path;

const HEADER: &[u8] = b"\x89PNG\r\n\x1a\n";

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct PngDimensions {
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) fn validate_png(path: &Path, expected: PngDimensions) -> Result<()> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read screenshot: {}", path.display()))?;
    anyhow::ensure!(
        bytes.len() >= 24 && bytes.starts_with(HEADER),
        "capture did not produce a complete PNG: {}",
        path.display()
    );
    anyhow::ensure!(
        bytes[8..12] == [0, 0, 0, 13] && &bytes[12..16] == b"IHDR",
        "capture is missing PNG IHDR: {}",
        path.display()
    );
    let actual = dimensions(&bytes);
    anyhow::ensure!(
        actual == expected && actual.width > 0 && actual.height > 0,
        "PNG dimensions {}x{} do not match window {}x{}",
        actual.width,
        actual.height,
        expected.width,
        expected.height
    );
    Ok(())
}

fn dimensions(bytes: &[u8]) -> PngDimensions {
    PngDimensions {
        width: u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        height: u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
    }
}

#[cfg(target_os = "linux")]
pub(super) fn remove_stale(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to clear stale capture: {}", path.display()))
        }
    }
}
