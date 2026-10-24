use std::{
    env,
    ffi::OsString,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Map, Value};
use tempfile::NamedTempFile;

const REAL_NPX: &str = "KCU_ACP_REAL_NPX";
const TRACE_DIR: &str = "KCU_ACP_RPC_TRACE_DIR";

fn main() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let adapter = adapter_name(&args);
    let (child, recorder) = launch(&args, adapter, trace_dir()?)?;
    forward(child, recorder)
}

fn launch(
    args: &[OsString],
    adapter: &'static str,
    trace_dir: Option<PathBuf>,
) -> Result<(std::process::Child, Option<Recorder>), String> {
    let mut child = Command::new(real_npx()?)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to launch ACP adapter: {error}"))?;
    let recorder = match trace_dir
        .map(|dir| Recorder::create(dir, adapter, child.id()))
        .transpose()
    {
        Ok(recorder) => recorder,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    Ok((child, recorder))
}

fn forward(mut child: std::process::Child, recorder: Option<Recorder>) -> Result<(), String> {
    let input = child
        .stdin
        .take()
        .ok_or("ACP adapter stdin is unavailable")?;
    let output = child
        .stdout
        .take()
        .ok_or("ACP adapter stdout is unavailable")?;
    let input_recorder = recorder.clone();
    let (_input_thread, input_result) = spawn_input(input, input_recorder);
    let output_recorder = recorder.clone();
    let output_thread = thread::spawn(move || copy_stdout(output, output_recorder));
    let status = child
        .wait()
        .map_err(|error| format!("ACP adapter wait failed: {error}"))?;
    output_thread
        .join()
        .map_err(|_| "ACP adapter stdout forwarding panicked".to_string())??;
    if let Ok(result) = input_result.try_recv() {
        result?;
    }
    if let Some(recorder) = recorder {
        recorder.finish()?;
    }
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("ACP adapter exited with {status}"))
}

fn spawn_input(
    output: std::process::ChildStdin,
    recorder: Option<Recorder>,
) -> (
    thread::JoinHandle<()>,
    std::sync::mpsc::Receiver<Result<(), String>>,
) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let thread = thread::spawn(move || {
        let _ = sender.send(copy_stdin(std::io::stdin(), output, recorder));
    });
    (thread, receiver)
}

fn copy_stdin(
    input: std::io::Stdin,
    mut output: std::process::ChildStdin,
    recorder: Option<Recorder>,
) -> Result<(), String> {
    let mut reader = BufReader::new(input.lock());
    let mut line = Vec::new();
    loop {
        let count = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("ACP client stdin read failed: {error}"))?;
        if count == 0 {
            return Ok(());
        }
        if let Some(recorder) = &recorder {
            recorder.record(&line, "client_to_agent");
        }
        output
            .write_all(&line)
            .and_then(|_| output.flush())
            .map_err(|error| format!("ACP adapter stdin write failed: {error}"))?;
        line.clear();
    }
}

fn copy_stdout(input: std::process::ChildStdout, recorder: Option<Recorder>) -> Result<(), String> {
    let mut reader = BufReader::new(input);
    let mut output = std::io::stdout().lock();
    let mut line = Vec::new();
    loop {
        let count = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("ACP adapter stdout read failed: {error}"))?;
        if count == 0 {
            return output
                .flush()
                .map_err(|error| format!("ACP adapter stdout forwarding failed: {error}"));
        }
        if let Some(recorder) = &recorder {
            recorder.record(&line, "agent_to_client");
        }
        output
            .write_all(&line)
            .map_err(|error| format!("ACP adapter stdout forwarding failed: {error}"))?;
        line.clear();
    }
}

fn real_npx() -> Result<PathBuf, String> {
    let path =
        PathBuf::from(env::var_os(REAL_NPX).ok_or_else(|| format!("{REAL_NPX} is required"))?);
    path.is_absolute()
        .then_some(path)
        .ok_or_else(|| format!("{REAL_NPX} must be absolute"))
}

fn trace_dir() -> Result<Option<PathBuf>, String> {
    let Some(path) = env::var_os(TRACE_DIR) else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    if path.is_absolute() && path.is_dir() {
        Ok(Some(path))
    } else {
        Err(format!(
            "{TRACE_DIR} must be an existing absolute directory"
        ))
    }
}

fn adapter_name(args: &[OsString]) -> &'static str {
    if args
        .iter()
        .any(|arg| arg == "@agentclientprotocol/codex-acp@2.2.2")
    {
        "codex-acp"
    } else if args
        .iter()
        .any(|arg| arg == "@agentclientprotocol/claude-agent-acp@0.94.0")
    {
        "claude-agent-acp"
    } else {
        "unknown-acp"
    }
}

#[derive(Clone)]
struct Recorder {
    adapter: &'static str,
    child_pid: u32,
    trace: Arc<Mutex<TraceState>>,
    failure: Arc<Mutex<Option<String>>>,
    failure_marker: PathBuf,
}

struct TraceState {
    path: PathBuf,
    committed: Vec<u8>,
}

impl Recorder {
    fn create(dir: PathBuf, adapter: &'static str, child_pid: u32) -> Result<Self, String> {
        let path = dir.join(format!("{adapter}-{child_pid}.jsonl"));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("failed to create ACP trace: {error}"))?;
        drop(file);
        Ok(Self {
            adapter,
            child_pid,
            trace: Arc::new(Mutex::new(TraceState {
                path,
                committed: Vec::new(),
            })),
            failure: Arc::new(Mutex::new(None)),
            failure_marker: dir.join(format!("{adapter}-{child_pid}.failed")),
        })
    }

    fn record(&self, line: &[u8], direction: &str) {
        let Ok(value) = serde_json::from_slice::<Value>(line) else {
            self.fail("failed to parse outbound ACP JSON-RPC".to_string());
            return;
        };
        let method = value.get("method").and_then(Value::as_str);
        if method.is_none() && !is_response(&value) {
            self.fail("ACP JSON-RPC had no method or response".to_string());
            return;
        }
        let mut event = Map::new();
        event.insert("adapter".into(), Value::from(self.adapter));
        event.insert("child_pid".into(), Value::from(self.child_pid));
        event.insert("at_unix_ms".into(), Value::from(now_unix_ms()));
        event.insert("direction".into(), Value::from(direction));
        event.insert("method".into(), Value::from(method.unwrap_or("response")));
        if let Some(id) = value.get("id") {
            event.insert("id".into(), id.clone());
        }
        if let Some(error) = value.get("error") {
            event.insert("error".into(), error.clone());
        }
        self.write(Value::Object(event));
    }

    fn write(&self, event: Value) {
        let result = self.write_line(&event);
        if let Err(error) = result {
            self.fail(error);
        }
    }

    fn write_line(&self, event: &Value) -> Result<(), String> {
        let mut trace = self
            .trace
            .lock()
            .map_err(|_| "ACP trace lock poisoned".to_string())?;
        let mut encoded = serde_json::to_vec(event)
            .map_err(|error| format!("failed to encode ACP trace: {error}"))?;
        encoded.push(b'\n');
        // 会話破棄によるプロセス終了が書込みに割り込んでも、公開済み記録を保持する。
        let staged = stage_snapshot(&trace.path, &trace.committed, &encoded)?;
        staged
            .persist(&trace.path)
            .map_err(|error| format!("failed to publish ACP trace: {}", error.error))?;
        trace.committed.extend_from_slice(&encoded);
        Ok(())
    }

    fn fail(&self, error: String) {
        if let Ok(mut failure) = self.failure.lock()
            && failure.is_none()
        {
            *failure = Some(error);
            let _ = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.failure_marker)
                .and_then(|mut file| file.write_all(b"ACP trace failure\n"));
        }
    }

    fn finish(&self) -> Result<(), String> {
        self.failure
            .lock()
            .map_err(|_| "ACP trace state lock poisoned".to_string())?
            .clone()
            .map_or(Ok(()), Err)
    }
}

fn stage_snapshot(
    path: &Path,
    committed: &[u8],
    next_record: &[u8],
) -> Result<NamedTempFile, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "ACP trace has no parent directory".to_string())?;
    let mut staged = NamedTempFile::new_in(parent)
        .map_err(|error| format!("failed to stage ACP trace: {error}"))?;
    staged
        .write_all(committed)
        .and_then(|_| staged.write_all(next_record))
        .and_then(|_| staged.flush())
        .and_then(|_| staged.as_file().sync_all())
        .map_err(|error| format!("failed to stage ACP trace: {error}"))?;
    Ok(staged)
}

fn is_response(value: &Value) -> bool {
    value.get("id").is_some() && (value.get("result").is_some() || value.get("error").is_some())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().try_into().unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::{Recorder, adapter_name, stage_snapshot};
    use serde_json::Value;
    use std::{ffi::OsString, fs};

    #[test]
    fn recorder_identifies_the_actual_provider_launch_profiles()
    -> Result<(), Box<dyn std::error::Error>> {
        for (vendor, expected) in [
            ("codex-cli", "codex-acp"),
            ("claude-code", "claude-agent-acp"),
        ] {
            let profile = kcu_host_support::acp_provider_profile(vendor, std::env::current_dir()?)?;
            let args: Vec<OsString> = profile.args.into_iter().map(OsString::from).collect();
            assert_eq!(expected, adapter_name(&args));
        }
        Ok(())
    }

    #[test]
    fn recorder_publishes_only_complete_jsonl_snapshots() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = tempfile::tempdir()?;
        let recorder = Recorder::create(root.path().to_path_buf(), "test-acp", 42)?;
        let path = root.path().join("test-acp-42.jsonl");

        recorder.record(br#"{"jsonrpc":"2.0","method":"first"}"#, "client_to_agent");
        let previous = fs::read(&path)?;
        let staged = stage_snapshot(&path, &previous, br#"{"unfinished"#)?;
        drop(staged);
        assert_eq!(previous, fs::read(&path)?);

        recorder.record(br#"{"jsonrpc":"2.0","method":"second"}"#, "agent_to_client");
        let published = fs::read_to_string(&path)?;
        let records = published
            .lines()
            .map(serde_json::from_str::<Value>)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["method"], "first");
        assert_eq!(records[1]["method"], "second");
        Ok(())
    }
}
