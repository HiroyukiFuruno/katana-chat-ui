use super::metadata::HostMetadata;
use anyhow::{Context, Result, anyhow, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
};

const COMMAND_TIMEOUT_SECONDS: &str = "5";

pub(super) fn collect(metadata: &HostMetadata, child: &Child, output_dir: &Path) -> Result<()> {
    let diagnostic_dir = output_dir.join("failure-diagnostics");
    fs::create_dir_all(&diagnostic_dir)
        .with_context(|| format!("failed to create {}", diagnostic_dir.display()))?;
    let mut collector = Collector {
        directory: diagnostic_dir.clone(),
        failures: Vec::new(),
    };
    let child_pid = child.id();

    record_process_status(&mut collector, child_pid);
    record_display_and_wm(&mut collector);
    let window_id = collector.find_window(metadata, child_pid);
    record_window_details(&mut collector, window_id.as_deref());
    record_window_properties(&mut collector, window_id.as_deref());
    record_last_app_copy(
        &mut collector,
        output_dir,
        metadata.output_name,
        &diagnostic_dir,
    );
    record_last_app_capture(&mut collector, window_id.as_deref(), &diagnostic_dir);
    record_root_capture(&mut collector, &diagnostic_dir);
    collector.finish()
}

fn record_process_status(collector: &mut Collector, child_pid: u32) {
    collector.record("process-status.txt", || read_process_status(child_pid));
}

fn read_process_status(child_pid: u32) -> Result<String> {
    let pid = child_pid.to_string();
    let ps = command("ps", &["-o", "pid=,ppid=,stat=,wchan=,etime=", "-p", &pid])?;
    let status = read_proc_file(child_pid, "status")?;
    let stat = read_proc_file(child_pid, "stat")?;
    let wchan = read_proc_file(child_pid, "wchan")?;
    Ok(format!(
        "ps:\n{ps}\n/proc/status:\n{status}\n/proc/stat:\n{stat}\n/proc/wchan:\n{wchan}"
    ))
}

fn read_proc_file(child_pid: u32, file: &str) -> Result<String> {
    let path = format!("/proc/{child_pid}/{file}");
    fs::read_to_string(&path).with_context(|| format!("failed to read {path}"))
}

fn record_display_and_wm(collector: &mut Collector) {
    collector.record("display-and-wm.txt", read_display_and_wm);
}

fn read_display_and_wm() -> Result<String> {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| "<unset>".into());
    let root_properties = command("xprop", &["-root"])?;
    let root_tree = command("xwininfo", &["-root", "-tree"])?;
    Ok(format!(
        "DISPLAY={display}\nroot xprop:\n{root_properties}\nroot tree:\n{root_tree}"
    ))
}

fn record_window_details(collector: &mut Collector, window_id: Option<&str>) {
    collector.record("window.txt", || window_information(window_id, "xwininfo"));
}

fn record_window_properties(collector: &mut Collector, window_id: Option<&str>) {
    collector.record("window-xprop.txt", || {
        window_information(window_id, "xprop")
    });
}

fn window_information(window_id: Option<&str>, program: &str) -> Result<String> {
    let id = window_id.ok_or_else(|| anyhow!("matching window was not found"))?;
    command(program, &["-id", id])
}

fn record_last_app_copy(
    collector: &mut Collector,
    output_dir: &Path,
    output_name: &str,
    diagnostic_dir: &Path,
) {
    collector.record("last-app-copy.txt", || {
        copy_last_app(output_dir, output_name, diagnostic_dir)
    });
}

fn copy_last_app(output_dir: &Path, output_name: &str, diagnostic_dir: &Path) -> Result<String> {
    let source = output_dir.join(output_name);
    let destination = diagnostic_dir.join("last-app.png");
    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(format!(
        "source={} destination={}",
        source.display(),
        destination.display()
    ))
}

fn record_last_app_capture(
    collector: &mut Collector,
    window_id: Option<&str>,
    diagnostic_dir: &Path,
) {
    collector.record("last-app-capture.txt", || {
        capture_last_app(window_id, diagnostic_dir)
    });
}

fn capture_last_app(window_id: Option<&str>, diagnostic_dir: &Path) -> Result<String> {
    let id = window_id.ok_or_else(|| anyhow!("matching window was not found"))?;
    let xwd = diagnostic_dir.join("last-app-capture.xwd");
    let png = diagnostic_dir.join("last-app-capture.png");
    let capture_result = capture_app_files(id, &xwd, &png);
    combine_capture_and_cleanup(capture_result, &xwd)
}

fn capture_app_files(id: &str, xwd: &Path, png: &Path) -> Result<String> {
    run_command("xwd", &["-silent", "-id", id, "-out", &path_arg(xwd)])?;
    run_command("convert", &[&path_arg(xwd), &path_arg(png)])?;
    command(
        "identify",
        &["-format", "%m %wx%h colors=%k sha=%#\\n", &path_arg(png)],
    )
}

fn combine_capture_and_cleanup(capture_result: Result<String>, xwd: &Path) -> Result<String> {
    let cleanup_result = remove_xwd(xwd);
    match (capture_result, cleanup_result) {
        (Ok(identify), Ok(())) => Ok(identify),
        (Err(capture_error), Ok(())) => Err(capture_error),
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error),
        (Err(capture_error), Err(cleanup_error)) => {
            Err(anyhow!("{capture_error:#}; {cleanup_error:#}"))
        }
    }
}

fn remove_xwd(xwd: &Path) -> Result<()> {
    match fs::remove_file(xwd) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(anyhow!("failed to remove {}: {error}", xwd.display())),
    }
}

fn record_root_capture(collector: &mut Collector, diagnostic_dir: &Path) {
    collector.record("root-capture.txt", || capture_root(diagnostic_dir));
}

fn capture_root(diagnostic_dir: &Path) -> Result<String> {
    let png = diagnostic_dir.join("root.png");
    run_command("import", &["-window", "root", &path_arg(&png)])?;
    Ok(png.display().to_string())
}

struct Collector {
    directory: PathBuf,
    failures: Vec<String>,
}

impl Collector {
    fn record(&mut self, name: &str, operation: impl FnOnce() -> Result<String>) {
        let path = self.directory.join(name);
        let content = match operation() {
            Ok(value) => format!("status=ok\n{value}\n"),
            Err(error) => {
                self.failures.push(format!("{name}: {error:#}"));
                format!("status=error\nerror={error:#}\n")
            }
        };
        if let Err(error) = fs::write(&path, content) {
            self.failures
                .push(format!("{name}: failed to write result: {error}"));
        }
    }

    fn find_window(&mut self, metadata: &HostMetadata, child_pid: u32) -> Option<String> {
        let result = (|| -> Result<Option<String>> {
            let output = command("xdotool", &["search", "--name", metadata.window_title])?;
            for id in output.lines().map(str::trim).filter(|id| !id.is_empty()) {
                let title = command("xdotool", &["getwindowname", id])?;
                let pid = command("xprop", &["-id", id, "_NET_WM_PID"])?;
                if title.trim() == metadata.window_title
                    && pid
                        .split('=')
                        .nth(1)
                        .and_then(|value| value.trim().parse().ok())
                        == Some(child_pid)
                {
                    return Ok(Some(id.to_string()));
                }
            }
            Ok(None)
        })();
        match result {
            Ok(window) => window,
            Err(error) => {
                self.failures.push(format!("window search: {error:#}"));
                None
            }
        }
    }

    fn finish(self) -> Result<()> {
        if self.failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "failure diagnostics had errors: {}",
                self.failures.join("; ")
            ))
        }
    }
}

fn command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("timeout")
        .args([COMMAND_TIMEOUT_SECONDS, program])
        .args(args)
        .output()
        .with_context(|| format!("failed to run {program}"))?;
    ensure!(
        output.status.success(),
        "{program} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_command(program: &str, args: &[&str]) -> Result<()> {
    command(program, args).map(|_| ())
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
