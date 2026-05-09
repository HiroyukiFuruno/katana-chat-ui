use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const WINDOW_WAIT_SECONDS: u64 = 30;
const WINDOW_POLL_MILLIS: u64 = 350;
const CAPTURE_STABILIZE_MILLIS: u64 = 700;
const CAPTURE_RETRY_COUNT: usize = 5;

const WINDOW_QUERY_SCRIPT: &str = r#"
import sys
try:
    import Quartz
except Exception:
    sys.exit(2)

owner = sys.argv[1]
title = sys.argv[2]
candidates = []
windows = Quartz.CGWindowListCopyWindowInfo(
    Quartz.kCGWindowListOptionAll,
    Quartz.kCGNullWindowID
)
for window in windows:
    layer = window.get("kCGWindowLayer", 999)
    owner_name = str(window.get("kCGWindowOwnerName") or "")
    window_title = str(window.get("kCGWindowName") or "")
    bounds = window.get("kCGWindowBounds", {})
    width = bounds.get("Width", 0)
    height = bounds.get("Height", 0)
    if layer == 0 and width > 0 and height > 0 and (owner_name == owner or window_title == title):
        title_score = 1 if window_title == title else 0
        owner_score = 1 if owner_name == owner else 0
        candidates.append((title_score, owner_score, width * height, window.get("kCGWindowNumber")))
if candidates:
    candidates.sort(reverse=True)
    print(candidates[0][3])
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum NativeHost {
    Egui,
    Floem,
    Gpui,
}

pub fn run(host: NativeHost, output_dir: &Path, expected_hash: Option<&str>) -> Result<()> {
    if !cfg!(target_os = "macos") {
        bail!("native host screenshot is currently supported only on macOS");
    }

    let metadata = HostMetadata::for_host(host);
    let root = repo_root()?;
    metadata.build(&root)?;

    let mut child = metadata.spawn(&root)?;
    let capture_result = capture_native_window(&metadata, output_dir, expected_hash);
    stop_child(&mut child);
    capture_result
}

fn capture_native_window(
    metadata: &HostMetadata,
    output_dir: &Path,
    expected_hash: Option<&str>,
) -> Result<()> {
    let window_id = wait_for_window_id(metadata)?;
    thread::sleep(Duration::from_millis(CAPTURE_STABILIZE_MILLIS));

    let output_path = output_dir.join(metadata.output_name);
    run_screencapture(window_id, &output_path)?;
    let actual_hash = image_hash(&output_path)?;
    println!(
        "[kcu-screenshot] native {}: {}",
        metadata.host_label,
        output_path.display()
    );
    println!("[kcu-screenshot] native sha256: {actual_hash}");
    validate_hash(&output_path, expected_hash, &actual_hash)
}

fn run_screencapture(window_id: u64, output_path: &Path) -> Result<()> {
    let window_arg = window_id.to_string();
    let output_arg = output_path.display().to_string();
    for attempt_index in 0..CAPTURE_RETRY_COUNT {
        let output = Command::new("screencapture")
            .args(["-x", "-l", &window_arg, &output_arg])
            .output()
            .context("failed to launch screencapture")?;
        if output.status.success() {
            return Ok(());
        }
        let error_message = format!(
            "{} {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
        if attempt_index + 1 < CAPTURE_RETRY_COUNT {
            thread::sleep(Duration::from_millis(WINDOW_POLL_MILLIS));
        } else {
            anyhow::bail!("screencapture failed for window {window_arg}: {error_message}");
        }
    }
    Ok(())
}

fn validate_hash(output_path: &Path, expected: Option<&str>, actual: &str) -> Result<()> {
    if let Some(expected) = expected {
        anyhow::ensure!(
            expected == actual,
            "native screenshot baseline mismatch for {}: expected {}, actual {}",
            output_path.display(),
            expected,
            actual
        );
    }
    Ok(())
}

fn image_hash(output_path: &Path) -> Result<String> {
    let bytes = std::fs::read(output_path)
        .with_context(|| format!("failed to read screenshot: {}", output_path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn wait_for_window_id(metadata: &HostMetadata) -> Result<u64> {
    let deadline = Instant::now() + Duration::from_secs(WINDOW_WAIT_SECONDS);
    loop {
        if let Some(window_id) = query_window_id(metadata) {
            return Ok(window_id);
        }
        if Instant::now() >= deadline {
            bail!(
                "native host window not found: owner={}, title={}",
                metadata.owner_name,
                metadata.window_title
            );
        }
        thread::sleep(Duration::from_millis(WINDOW_POLL_MILLIS));
    }
}

fn query_window_id(metadata: &HostMetadata) -> Option<u64> {
    let output = Command::new("python3")
        .arg("-c")
        .arg(WINDOW_QUERY_SCRIPT)
        .arg(metadata.owner_name)
        .arg(metadata.window_title)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .and_then(|line| line.trim().parse::<u64>().ok())
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

#[derive(Clone, Copy)]
struct HostMetadata {
    host_label: &'static str,
    manifest_path: &'static str,
    binary_name: &'static str,
    owner_name: &'static str,
    window_title: &'static str,
    output_name: &'static str,
}

impl HostMetadata {
    fn for_host(host: NativeHost) -> Self {
        match host {
            NativeHost::Egui => Self::egui(),
            NativeHost::Floem => Self::floem(),
            NativeHost::Gpui => Self::gpui(),
        }
    }

    fn egui() -> Self {
        Self {
            host_label: "egui",
            manifest_path: "tools/manual-host-egui/Cargo.toml",
            binary_name: "manual-host-egui",
            owner_name: "manual-host-egui",
            window_title: "katana-chat-ui harness egui",
            output_name: "native-egui.png",
        }
    }

    fn floem() -> Self {
        Self {
            host_label: "floem",
            manifest_path: "tools/manual-host-floem/Cargo.toml",
            binary_name: "manual-host-floem",
            owner_name: "manual-host-floem",
            window_title: "katana-chat-ui harness floem",
            output_name: "native-floem.png",
        }
    }

    fn gpui() -> Self {
        Self {
            host_label: "gpui",
            manifest_path: "tools/manual-host-gpui/Cargo.toml",
            binary_name: "manual-host-gpui",
            owner_name: "manual-host-gpui",
            window_title: "katana-chat-ui harness gpui",
            output_name: "native-gpui.png",
        }
    }

    fn build(self, root: &Path) -> Result<()> {
        let status = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(root.join(self.manifest_path))
            .arg("--locked")
            .env("CARGO_TARGET_DIR", self.target_dir(root))
            .status()
            .with_context(|| format!("failed to build {}", self.host_label))?;
        anyhow::ensure!(
            status.success(),
            "cargo build failed for {}",
            self.host_label
        );
        Ok(())
    }

    fn spawn(self, root: &Path) -> Result<Child> {
        Command::new(self.binary_path(root))
            .current_dir(root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to launch {}", self.host_label))
    }

    fn target_dir(self, root: &Path) -> PathBuf {
        root.join("target")
            .join("native-harness")
            .join(self.host_label)
    }

    fn binary_path(self, root: &Path) -> PathBuf {
        self.target_dir(root).join("debug").join(self.binary_name)
    }
}

#[cfg(test)]
mod tests {
    use super::{HostMetadata, NativeHost, WINDOW_QUERY_SCRIPT};

    #[test]
    fn native_hosts_have_stable_window_titles() {
        assert_eq!(
            HostMetadata::for_host(NativeHost::Egui).window_title,
            "katana-chat-ui harness egui"
        );
        assert_eq!(
            HostMetadata::for_host(NativeHost::Floem).window_title,
            "katana-chat-ui harness floem"
        );
        assert_eq!(
            HostMetadata::for_host(NativeHost::Gpui).window_title,
            "katana-chat-ui harness gpui"
        );
    }

    #[test]
    fn native_hosts_write_separate_output_files() {
        assert_eq!(
            HostMetadata::for_host(NativeHost::Egui).output_name,
            "native-egui.png"
        );
        assert_eq!(
            HostMetadata::for_host(NativeHost::Floem).output_name,
            "native-floem.png"
        );
        assert_eq!(
            HostMetadata::for_host(NativeHost::Gpui).output_name,
            "native-gpui.png"
        );
    }

    #[test]
    fn window_query_prefers_titled_application_window() {
        assert!(WINDOW_QUERY_SCRIPT.contains("Quartz.kCGWindowListOptionAll"));
        assert!(WINDOW_QUERY_SCRIPT.contains("title_score = 1 if window_title == title else 0"));
        assert!(WINDOW_QUERY_SCRIPT.contains("candidates.sort(reverse=True)"));
    }
}
