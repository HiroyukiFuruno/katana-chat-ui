use std::{fs, path::PathBuf};

use katana_chat_ui::ChatSession;

use super::{ChatHistoryRecord, ChatHistoryStore};

#[test]
fn ds_store_is_ignored_while_a_real_record_remains_readable() -> Result<(), String> {
    let root = test_root("ds-store");
    let store = ChatHistoryStore::new(root.clone());
    store.save("katanagent", "session-1", ChatSession::new().snapshot())?;
    let sessions = root.join("tmp/harness-katanagent/sessions");
    fs::write(sessions.join(".DS_Store"), "finder metadata").map_err(|error| error.to_string())?;

    assert_eq!(store.list()?.len(), 1);
    cleanup(root)
}

#[test]
fn malformed_json_is_reported_instead_of_being_silently_skipped() -> Result<(), String> {
    let root = test_root("bad-json");
    let store = ChatHistoryStore::new(root.clone());
    store.save("katanagent", "session-1", ChatSession::new().snapshot())?;
    fs::write(
        root.join("tmp/harness-katanagent/sessions/bad.json"),
        "not json",
    )
    .map_err(|error| error.to_string())?;

    assert!(store.list().is_err());
    cleanup(root)
}

#[test]
fn unknown_record_version_is_rejected() -> Result<(), String> {
    let root = test_root("unknown-version");
    let store = ChatHistoryStore::new(root.clone());
    let record = ChatHistoryRecord {
        version: 2,
        session_id: "session-2".to_string(),
        provider_id: "katanagent".to_string(),
        updated_at_unix_millis: 1,
        snapshot: ChatSession::new().snapshot(),
    };
    let path = root.join("tmp/harness-katanagent/sessions/session-2.json");
    fs::create_dir_all(path.parent().ok_or("missing parent")?)
        .map_err(|error| error.to_string())?;
    fs::write(
        path,
        serde_json::to_string(&record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    assert!(store.list().is_err());
    cleanup(root)
}

fn test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kcu-history-{label}-{}",
        ChatHistoryStore::new_session_id()
    ))
}

fn cleanup(path: PathBuf) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}
