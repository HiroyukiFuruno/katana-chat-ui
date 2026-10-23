use super::{
    metadata::HostMetadata,
    png::{PngDimensions, validate_png},
    wait_until,
};
use anyhow::{Context, Result};
use std::{
    path::Path,
    process::{Child, Command},
};

const WINDOW_QUERY_SCRIPT: &str = r#"
import sys
try:
    import Quartz
except Exception:
    sys.exit(2)

child_pid, title = int(sys.argv[1]), sys.argv[2]
candidates = []
for window in Quartz.CGWindowListCopyWindowInfo(Quartz.kCGWindowListOptionAll, Quartz.kCGNullWindowID):
    window_title = str(window.get("kCGWindowName") or "")
    window_id = window.get("kCGWindowNumber")
    if window.get("kCGWindowLayer", 999) != 0 or window_title != title or window.get("kCGWindowOwnerPID") != child_pid or not window_id:
        continue
    image = Quartz.CGWindowListCreateImage(
        Quartz.CGRectNull,
        Quartz.kCGWindowListOptionIncludingWindow,
        window_id,
        Quartz.kCGWindowImageBoundsIgnoreFraming | Quartz.kCGWindowImageBestResolution,
    )
    if image:
        candidates.append((Quartz.CGImageGetWidth(image) * Quartz.CGImageGetHeight(image), window_id, Quartz.CGImageGetWidth(image), Quartz.CGImageGetHeight(image)))
if candidates:
    _, window_id, width, height = sorted(candidates, reverse=True)[0]
    print(window_id, width, height)
"#;

pub(super) fn capture(
    metadata: &HostMetadata,
    child: &mut Child,
    output_path: &Path,
) -> Result<()> {
    let child_pid = child.id();
    let window = wait_until(child, "macOS native window", |_| {
        query_window(metadata.window_title, child_pid)
    })?;
    capture_window(window, output_path)?;
    validate_png(output_path, window.dimensions)
}

fn query_window(title: &str, child_pid: u32) -> Result<Option<Window>> {
    let output = Command::new("python3")
        .args(["-c", WINDOW_QUERY_SCRIPT, &child_pid.to_string(), title])
        .output()
        .context("failed to query macOS windows")?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(parse_window(&String::from_utf8_lossy(&output.stdout)))
}

fn capture_window(window: Window, output_path: &Path) -> Result<()> {
    let output = Command::new("screencapture")
        .args(["-x", "-o", "-l", &window.id.to_string()])
        .arg(output_path)
        .output()
        .context("failed to launch screencapture")?;
    anyhow::ensure!(
        output.status.success(),
        "screencapture failed for window {}: {}",
        window.id,
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct Window {
    id: u64,
    dimensions: PngDimensions,
}

fn parse_window(output: &str) -> Option<Window> {
    let values = output
        .split_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    match values.as_slice() {
        [id, width, height] if *width > 0 && *height > 0 => Some(Window {
            id: *id,
            dimensions: PngDimensions {
                width: u32::try_from(*width).ok()?,
                height: u32::try_from(*height).ok()?,
            },
        }),
        _ => None,
    }
}
