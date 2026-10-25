#[cfg(target_os = "linux")]
mod failure_diagnostics;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod metadata;
mod png;

#[cfg(test)]
mod tests;

use crate::{hash::sha256_hex, native_fixture::PreparedNativeFixture};
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use metadata::HostMetadata;
use std::{
    path::{Path, PathBuf},
    process::Child,
    thread,
    time::{Duration, Instant},
};

const WINDOW_WAIT_SECONDS: u64 = 30;
const WINDOW_POLL_MILLIS: u64 = 350;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum NativeHost {
    Egui,
    Floem,
    Gpui,
}

pub fn run(
    host: NativeHost,
    output_dir: &Path,
    expected_hash: Option<&str>,
    fixture: Option<&PreparedNativeFixture>,
) -> Result<()> {
    let metadata = HostMetadata::for_host(host);
    let root = repo_root()?;
    metadata.build(&root)?;
    #[cfg(target_os = "linux")]
    linux::confirm_headless_display()?;
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    bail!("native host screenshot is unsupported on this platform");
    let mut child = metadata.spawn(&root, fixture)?;
    let capture_result = capture_native_window(&metadata, &mut child, output_dir, expected_hash);
    #[cfg(target_os = "linux")]
    let diagnostics_result = if capture_result.is_err() {
        Some(failure_diagnostics::collect(&metadata, &child, output_dir))
    } else {
        None
    };
    stop_child(&mut child);
    #[cfg(target_os = "linux")]
    if let (Err(_capture_error), Some(Err(diagnostics_error))) =
        (capture_result.as_ref(), diagnostics_result.as_ref())
    {
        return capture_result.context(format!(
            "failure diagnostics also failed: {diagnostics_error:#}"
        ));
    }
    capture_result
}

fn capture_native_window(
    metadata: &HostMetadata,
    child: &mut Child,
    output_dir: &Path,
    expected_hash: Option<&str>,
) -> Result<()> {
    let output_path = output_dir.join(metadata.output_name);
    let actual_hash = capture_mature_frame(metadata, child, &output_path, expected_hash)?;
    println!(
        "[kcu-screenshot] native {}: {}",
        metadata.host_label,
        output_path.display()
    );
    println!("[kcu-screenshot] native sha256: {actual_hash}");
    if expected_hash.is_none() {
        println!("[kcu-screenshot] native capture is an artifact only, not a 0015 gate");
    }
    Ok(())
}

fn capture_mature_frame(
    metadata: &HostMetadata,
    child: &mut Child,
    output_path: &Path,
    expected_hash: Option<&str>,
) -> Result<String> {
    match expected_hash {
        Some(expected_hash) => {
            capture_matching_baseline(metadata, child, output_path, expected_hash)
        }
        None => capture_once(metadata, child, output_path),
    }
}

fn capture_matching_baseline(
    metadata: &HostMetadata,
    child: &mut Child,
    output_path: &Path,
    expected_hash: &str,
) -> Result<String> {
    let mut last_hash = None;
    let result = wait_until(child, "native screenshot matching its baseline", |child| {
        let actual_hash = capture_once(metadata, child, output_path)?;
        last_hash = Some(actual_hash.clone());
        Ok((actual_hash == expected_hash).then_some(actual_hash))
    });
    result.with_context(|| {
        format!(
            "expected {expected_hash}; last capture was {}",
            last_hash.as_deref().map_or("unavailable", |hash| hash)
        )
    })
}

fn capture_once(metadata: &HostMetadata, child: &mut Child, output_path: &Path) -> Result<String> {
    capture_for_platform(metadata, child, output_path)?;
    image_hash(output_path)
}

#[cfg(target_os = "macos")]
fn capture_for_platform(
    metadata: &HostMetadata,
    child: &mut Child,
    output_path: &Path,
) -> Result<()> {
    macos::capture(metadata, child, output_path)
}

#[cfg(target_os = "linux")]
fn capture_for_platform(
    metadata: &HostMetadata,
    child: &mut Child,
    output_path: &Path,
) -> Result<()> {
    linux::capture(metadata, child, output_path)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn capture_for_platform(
    _metadata: &HostMetadata,
    _child: &mut Child,
    _output_path: &Path,
) -> Result<()> {
    bail!("native host screenshot is unsupported on this platform")
}

fn image_hash(output_path: &Path) -> Result<String> {
    let bytes = std::fs::read(output_path)
        .with_context(|| format!("failed to read screenshot: {}", output_path.display()))?;
    Ok(sha256_hex(&bytes))
}

pub(super) fn wait_until<T>(
    child: &mut Child,
    label: &str,
    query: impl FnMut(&mut Child) -> Result<Option<T>>,
) -> Result<T> {
    let deadline = Instant::now() + Duration::from_secs(WINDOW_WAIT_SECONDS);
    poll_until(child, label, deadline, query)
}

fn poll_until<T>(
    child: &mut Child,
    label: &str,
    deadline: Instant,
    mut query: impl FnMut(&mut Child) -> Result<Option<T>>,
) -> Result<T> {
    loop {
        ensure_child_running(child, label)?;
        if let Some(value) = query(child)? {
            return Ok(value);
        }
        if Instant::now() >= deadline {
            bail!("timed out waiting for {label}");
        }
        thread::sleep(Duration::from_millis(WINDOW_POLL_MILLIS));
    }
}

fn ensure_child_running(child: &mut Child, label: &str) -> Result<()> {
    if let Some(status) = child
        .try_wait()
        .context("failed to inspect native host process")?
    {
        bail!("native host exited while waiting for {label}: {status}");
    }
    Ok(())
}

fn stop_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(root) = manifest_dir.parent().and_then(Path::parent) else {
        bail!("failed to resolve repository root from scripts/screenshot");
    };
    Ok(root.to_path_buf())
}
