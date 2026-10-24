use super::NativeHost;
use crate::native_fixture::PreparedNativeFixture;
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

#[derive(Clone, Copy)]
pub(super) struct HostMetadata {
    pub(super) host_label: &'static str,
    manifest_path: &'static str,
    binary_name: &'static str,
    pub(super) window_title: &'static str,
    pub(super) output_name: &'static str,
}

impl HostMetadata {
    pub(super) fn for_host(host: NativeHost) -> Self {
        match host {
            NativeHost::Egui => Self::egui(),
            NativeHost::Floem => Self::floem(),
            NativeHost::Gpui => Self::gpui(),
        }
    }

    fn egui() -> Self {
        Self::from_parts(
            "egui",
            "tools/manual-host-egui/Cargo.toml",
            "manual-host-egui",
            "katana-chat-ui harness egui",
            "native-egui.png",
        )
    }

    fn floem() -> Self {
        Self::from_parts(
            "floem",
            "tools/manual-host-floem/Cargo.toml",
            "manual-host-floem",
            "katana-chat-ui harness floem",
            "native-floem.png",
        )
    }

    fn gpui() -> Self {
        Self::from_parts(
            "gpui",
            "tools/manual-host-gpui/Cargo.toml",
            "manual-host-gpui",
            "katana-chat-ui harness gpui",
            "native-gpui.png",
        )
    }

    fn from_parts(
        host_label: &'static str,
        manifest_path: &'static str,
        binary_name: &'static str,
        window_title: &'static str,
        output_name: &'static str,
    ) -> Self {
        Self {
            host_label,
            manifest_path,
            binary_name,
            window_title,
            output_name,
        }
    }

    pub(super) fn build(self, root: &Path) -> Result<()> {
        let status = Command::new("cargo")
            .current_dir(root)
            .args(["build", "--manifest-path"])
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

    pub(super) fn spawn(
        self,
        root: &Path,
        fixture: Option<&PreparedNativeFixture>,
    ) -> Result<Child> {
        let mut command = Command::new(self.binary_path(root));
        command
            .current_dir(root)
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        if let Some(fixture) = fixture {
            command.args(["--kcu-surface"]).arg(&fixture.surface_path);
            command.args([
                "--kcu-viewport",
                &fixture.viewport.width.to_string(),
                &fixture.viewport.height.to_string(),
            ]);
        }
        command
            .spawn()
            .with_context(|| format!("failed to launch {}", self.host_label))
    }

    fn target_dir(self, root: &Path) -> PathBuf {
        std::env::var_os("CARGO_TARGET_DIR").map_or_else(
            || root.join("target/native-harness").join(self.host_label),
            |path| root.join(path),
        )
    }

    fn binary_path(self, root: &Path) -> PathBuf {
        self.target_dir(root).join("debug").join(self.binary_name)
    }
}
