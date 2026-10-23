use super::TraceWriter;
use kcu_host_support::ManagedHostCommand;
use std::{
    fs::{File, OpenOptions},
    sync::{Arc, Mutex, atomic::AtomicBool},
};

#[test]
fn command_log_preserves_unicode_and_line_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("kcu-trace-{}-{stamp}.jsonl", std::process::id()));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let trace = writer(file);
    trace.command(
        "egui",
        7,
        &ManagedHostCommand::DraftChanged("日本語\n次の行".to_string()),
    );
    assert!(!trace.failed());
    let text = std::fs::read_to_string(&path)?;
    assert_eq!(text.lines().count(), 1);
    let event: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(event["seq"], 7);
    assert_eq!(event["command"], "DraftChanged(\"日本語\\n次の行\")");
    drop(trace);
    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn write_failure_is_sticky_without_failing_the_command_caller()
-> Result<(), Box<dyn std::error::Error>> {
    let source = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))?;
    let trace = writer(source);
    trace.command("egui", 1, &ManagedHostCommand::DraftChanged("text".into()));
    assert!(trace.failed());
    trace.command("egui", 2, &ManagedHostCommand::DraftChanged("next".into()));
    assert!(trace.failed());
    Ok(())
}

fn writer(file: File) -> TraceWriter {
    TraceWriter {
        file: Arc::new(Mutex::new(file)),
        failure: Arc::new(Mutex::new(None)),
        reported: Arc::new(AtomicBool::new(false)),
    }
}
