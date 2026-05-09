use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use katana_chat_ui::ChatSessionSnapshot;
use serde::{Deserialize, Serialize};

const HISTORY_VERSION: u16 = 1;
const HARNESS_PREFIX: &str = "harness-";
const SESSIONS_DIR: &str = "sessions";
const LAST_SESSION_FILE: &str = "last-session.json";
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct ManualHistoryStore {
    cwd: PathBuf,
}

impl ManualHistoryStore {
    pub(crate) fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    pub(crate) fn new_session_id() -> String {
        let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("session-{}-{counter}", unix_millis())
    }

    pub(crate) fn save(
        &self,
        provider_id: &str,
        session_id: &str,
        snapshot: ChatSessionSnapshot,
    ) -> Result<(), String> {
        let record = ManualHistoryRecord::new(provider_id, session_id, snapshot);
        let root = self.provider_root(provider_id);
        let sessions_root = root.join(SESSIONS_DIR);
        fs::create_dir_all(&sessions_root).map_err(|error| error.to_string())?;
        self.write_record(&sessions_root.join(format!("{session_id}.json")), &record)?;
        self.write_record(&root.join(LAST_SESSION_FILE), &record)
    }

    pub(crate) fn load_latest(&self) -> Result<Option<ManualHistoryRecord>, String> {
        let tmp_root = self.cwd.join("tmp");
        if !tmp_root.is_dir() {
            return Ok(None);
        }
        let mut latest = None;
        for provider_root in fs::read_dir(tmp_root).map_err(|error| error.to_string())? {
            let provider_root = provider_root.map_err(|error| error.to_string())?;
            self.load_provider_latest(provider_root.path(), &mut latest)?;
        }
        Ok(latest)
    }

    fn load_provider_latest(
        &self,
        provider_root: PathBuf,
        latest: &mut Option<ManualHistoryRecord>,
    ) -> Result<(), String> {
        if !is_harness_root(&provider_root) {
            return Ok(());
        }
        let sessions_root = provider_root.join(SESSIONS_DIR);
        if !sessions_root.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(sessions_root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let record = Self::read_record(&entry.path())?;
            if latest_record_is_older(latest, &record) {
                *latest = Some(record);
            }
        }
        Ok(())
    }

    fn provider_root(&self, provider_id: &str) -> PathBuf {
        self.cwd
            .join("tmp")
            .join(format!("{HARNESS_PREFIX}{}", safe_provider_id(provider_id)))
    }

    fn write_record(&self, path: &Path, record: &ManualHistoryRecord) -> Result<(), String> {
        let content = serde_json::to_string_pretty(record).map_err(|error| error.to_string())?;
        fs::write(path, content).map_err(|error| error.to_string())
    }

    fn read_record(path: &Path) -> Result<ManualHistoryRecord, String> {
        let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
        serde_json::from_str(&content).map_err(|error| format!("{}: {error}", path.display()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManualHistoryRecord {
    pub(crate) version: u16,
    pub(crate) session_id: String,
    pub(crate) provider_id: String,
    pub(crate) updated_at_unix_millis: u128,
    pub(crate) snapshot: ChatSessionSnapshot,
}

impl ManualHistoryRecord {
    fn new(provider_id: &str, session_id: &str, snapshot: ChatSessionSnapshot) -> Self {
        Self {
            version: HISTORY_VERSION,
            session_id: session_id.to_string(),
            provider_id: provider_id.to_string(),
            updated_at_unix_millis: unix_millis(),
            snapshot,
        }
    }
}

fn latest_record_is_older(
    latest: &Option<ManualHistoryRecord>,
    record: &ManualHistoryRecord,
) -> bool {
    match latest {
        Some(latest) => latest.updated_at_unix_millis < record.updated_at_unix_millis,
        None => true,
    }
}

fn is_harness_root(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .and_then(|it| it.to_str())
            .is_some_and(|name| name.starts_with(HARNESS_PREFIX))
}

fn safe_provider_id(provider_id: &str) -> String {
    provider_id
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' => character,
            _ => '-',
        })
        .collect()
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::ManualHistoryStore;
    use katana_chat_ui::ChatSession;
    use std::{fs, path::PathBuf};

    #[test]
    fn saves_session_under_provider_specific_harness_directory() -> Result<(), String> {
        let root = temp_root("provider-root");
        let store = ManualHistoryStore::new(root.clone());
        let session = ChatSession::new();

        store.save("katanagent", "session-1", session.snapshot())?;

        let saved_path = root
            .join("tmp")
            .join("harness-katanagent")
            .join("sessions")
            .join("session-1.json");
        assert!(saved_path.is_file());
        cleanup(root);
        Ok(())
    }

    #[test]
    fn loads_latest_session_from_all_provider_histories() -> Result<(), String> {
        let root = temp_root("latest");
        let store = ManualHistoryStore::new(root.clone());
        store.save("katanagent", "session-1", titled_snapshot("old"))?;
        store.save("claude-code", "session-2", titled_snapshot("new"))?;

        let latest = store
            .load_latest()?
            .ok_or_else(|| "latest history is missing".to_string())?;

        assert_eq!(latest.session_id, "session-2");
        assert_eq!(latest.snapshot.title, "new");
        cleanup(root);
        Ok(())
    }

    fn titled_snapshot(title: &str) -> katana_chat_ui::ChatSessionSnapshot {
        let mut session = ChatSession::new();
        session.set_title(title);
        session.snapshot()
    }

    fn temp_root(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "kcu-history-{label}-{}",
            ManualHistoryStore::new_session_id()
        ));
        let _ = fs::remove_dir_all(&path);
        path
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
