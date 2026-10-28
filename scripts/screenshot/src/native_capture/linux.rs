use super::{
    metadata::HostMetadata,
    png::{PngDimensions, remove_stale, validate_png},
    wait_until,
};
use anyhow::{Context, Result};

mod readiness;

use readiness::visible_pixels;
use std::{
    env,
    path::Path,
    process::{Child, Command},
};

const HEADLESS_ENV: &str = "KCU_X11_HEADLESS";

pub(super) fn capture(
    metadata: &HostMetadata,
    child: &mut Child,
    output_path: &Path,
) -> Result<()> {
    let child_pid = child.id();
    let window = wait_until(child, "Linux X11 native window", |_| {
        find_window(metadata.window_title, child_pid)
    })?;
    // 非単色は描画開始の判定だけで、品質は呼出側のhash gateに委ねる。
    wait_until(
        child,
        "Linux X11 native frame with non-uniform pixels",
        |_| {
            capture_window(child_pid, window, output_path)?;
            Ok(visible_pixels(output_path)?.then_some(()))
        },
    )
}

pub(super) fn confirm_headless_display() -> Result<()> {
    anyhow::ensure!(
        env::var(HEADLESS_ENV).as_deref() == Ok("1"),
        "Linux native capture requires {HEADLESS_ENV}=1 to confirm an isolated headless X11 display"
    );
    let display = env::var("DISPLAY").context("Linux native capture requires DISPLAY")?;
    anyhow::ensure!(
        !display.is_empty(),
        "Linux native capture requires a non-empty DISPLAY"
    );
    let status = Command::new("xdpyinfo")
        .arg("-display")
        .arg(display)
        .status()
        .context("failed to inspect DISPLAY with xdpyinfo")?;
    anyhow::ensure!(status.success(), "DISPLAY is not an available X11 display");
    Ok(())
}

fn find_window(title: &str, child_pid: u32) -> Result<Option<Window>> {
    let output = Command::new("xdotool")
        .args(["search", "--name", title])
        .output()
        .context("failed to query X11 windows with xdotool")?;
    if !output.status.success() {
        anyhow::ensure!(
            output.stderr.is_empty(),
            "xdotool failed while searching for {title}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return Ok(None);
    }
    for id in parse_ids(&output.stdout)? {
        if title_matches(id, title)?
            && window_pid(id)? == Some(child_pid)
            && let Some(window) = window_details(id)?
        {
            return Ok(Some(window));
        }
    }
    Ok(None)
}

fn parse_ids(output: &[u8]) -> Result<Vec<u64>> {
    let ids = String::from_utf8_lossy(output)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim()
                .parse()
                .context("xdotool returned an invalid window ID")
        })
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(!ids.is_empty(), "xdotool returned no window IDs");
    Ok(ids)
}

fn title_matches(id: u64, title: &str) -> Result<bool> {
    let output = Command::new("xdotool")
        .args(["getwindowname", &id.to_string()])
        .output()
        .context("failed to read X11 window title")?;
    Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == title)
}

fn window_pid(id: u64) -> Result<Option<u32>> {
    let output = Command::new("xprop")
        .args(["-id", &id.to_string(), "_NET_WM_PID"])
        .output()
        .context("failed to read X11 window PID")?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split('=')
        .nth(1)
        .and_then(|value| value.trim().parse().ok()))
}

fn window_details(id: u64) -> Result<Option<Window>> {
    let output = Command::new("xwininfo")
        .args(["-id", &id.to_string()])
        .output()
        .context("failed to read X11 window dimensions")?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let width = field(&text, "Width:");
    let height = field(&text, "Height:");
    let viewable = text
        .lines()
        .any(|line| line.trim() == "Map State: IsViewable");
    Ok(width.zip(height).and_then(|(width, height)| {
        (width > 0 && height > 0 && viewable).then_some(Window { id, width, height })
    }))
}

fn field(text: &str, name: &str) -> Option<u32> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix(name)?.trim().parse().ok())
}

fn capture_window(child_pid: u32, window: Window, output_path: &Path) -> Result<()> {
    anyhow::ensure!(
        window.width > 0 && window.height > 0,
        "X11 window has invalid dimensions"
    );
    let xwd_path = output_path.with_extension(format!("{child_pid}.xwd"));
    let png_path = output_path.with_extension(format!("{child_pid}.png"));
    remove_stale(&xwd_path)?;
    remove_stale(&png_path)?;
    run_xwd(window.id, &xwd_path)?;
    run_convert(&xwd_path, &png_path)?;
    validate_png(
        &png_path,
        PngDimensions {
            width: window.width,
            height: window.height,
        },
    )?;
    std::fs::rename(&png_path, output_path)
        .with_context(|| format!("failed to publish screenshot: {}", output_path.display()))?;
    std::fs::remove_file(&xwd_path)
        .with_context(|| format!("failed to remove capture data: {}", xwd_path.display()))
}

fn run_xwd(window_id: u64, output_path: &Path) -> Result<()> {
    let output = Command::new("xwd")
        .args(["-silent", "-id", &window_id.to_string(), "-out"])
        .arg(output_path)
        .output()
        .context("failed to launch xwd")?;
    anyhow::ensure!(
        output.status.success(),
        "xwd failed for window {window_id}: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

fn run_convert(xwd_path: &Path, png_path: &Path) -> Result<()> {
    let output = Command::new("convert")
        .arg(xwd_path)
        .arg("-strip")
        .arg(png_path)
        .output()
        .context("failed to launch ImageMagick convert")?;
    anyhow::ensure!(
        output.status.success(),
        "ImageMagick convert failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct Window {
    id: u64,
    width: u32,
    height: u32,
}
