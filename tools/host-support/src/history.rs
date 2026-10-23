use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use katana_chat_ui::ChatSessionSnapshot;
use serde::{Deserialize, Serialize};

mod identifier;
use identifier::{safe_provider_id, validate_identifier};

#[cfg(test)]
mod tests;

const HISTORY_VERSION: u16 = 1;
const HARNESS_PREFIX: &str = "harness-";
const SESSIONS_DIRECTORY: &str = "sessions";
const LAST_SESSION_FILE: &str = "last-session.json";
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct ChatHistoryStore {
    root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatHistoryRecord {
    pub version: u16,
    pub session_id: String,
    pub provider_id: String,
    pub updated_at_unix_millis: u128,
    pub snapshot: ChatSessionSnapshot,
}

impl ChatHistoryStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn new_session_id() -> String {
        let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("session-{}-{counter}", unix_millis())
    }

    pub fn save(
        &self,
        provider_id: &str,
        session_id: &str,
        snapshot: ChatSessionSnapshot,
    ) -> Result<(), String> {
        validate_identifier(session_id)?;
        let record = ChatHistoryRecord::new(provider_id, session_id, snapshot);
        let root = self.provider_root(provider_id);
        let path = root
            .join(SESSIONS_DIRECTORY)
            .join(format!("{session_id}.json"));
        create_parent(&path)?;
        write_record(&path, &record)?;
        write_record(&root.join(LAST_SESSION_FILE), &record)
    }

    pub fn load_latest(&self) -> Result<Option<ChatHistoryRecord>, String> {
        Ok(self.list()?.into_iter().next())
    }

    pub fn load(&self, session_id: &str) -> Result<Option<ChatHistoryRecord>, String> {
        validate_identifier(session_id)?;
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.session_id == session_id))
    }

    pub fn list(&self) -> Result<Vec<ChatHistoryRecord>, String> {
        let tmp = self.root.join("tmp");
        if !tmp.is_dir() {
            return Ok(Vec::new());
        }
        let mut records = read_provider_records(&tmp)?;
        records.sort_by(newest_first);
        Ok(records)
    }

    fn provider_root(&self, provider_id: &str) -> PathBuf {
        self.root
            .join("tmp")
            .join(format!("{HARNESS_PREFIX}{}", safe_provider_id(provider_id)))
    }
}

impl ChatHistoryRecord {
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

fn create_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("history parent is unavailable: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())
}

fn write_record(path: &Path, record: &ChatHistoryRecord) -> Result<(), String> {
    let content = serde_json::to_string_pretty(record).map_err(|error| error.to_string())?;
    let parent = path.parent().ok_or("history parent is missing")?;
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    staged
        .write_all(content.as_bytes())
        .map_err(|error| error.to_string())?;
    staged.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}

fn read_provider_records(path: &Path) -> Result<Vec<ChatHistoryRecord>, String> {
    let mut records = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        records.extend(read_provider_entry(entry)?);
    }
    Ok(records)
}

fn read_provider_entry(
    entry: Result<fs::DirEntry, std::io::Error>,
) -> Result<Vec<ChatHistoryRecord>, String> {
    let path = entry.map_err(|error| error.to_string())?.path();
    if !is_harness_root(&path) {
        return Ok(Vec::new());
    }
    read_records(&path.join(SESSIONS_DIRECTORY))
}

fn read_records(path: &Path) -> Result<Vec<ChatHistoryRecord>, String> {
    if !path.is_dir() {
        return Ok(Vec::new());
    }
    fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .filter(|entry| {
            entry.as_ref().map_or(true, |entry| {
                entry.path().extension().is_some_and(|ext| ext == "json")
            })
        })
        .map(read_record)
        .collect()
}

fn read_record(entry: Result<fs::DirEntry, std::io::Error>) -> Result<ChatHistoryRecord, String> {
    let path = entry.map_err(|error| error.to_string())?.path();
    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let record: ChatHistoryRecord =
        serde_json::from_str(&content).map_err(|error| format!("{}: {error}", path.display()))?;
    if record.version != HISTORY_VERSION || record.snapshot.version != 1 {
        return Err(format!("unsupported history version: {}", path.display()));
    }
    validate_identifier(&record.session_id)?;
    Ok(record)
}

fn is_harness_root(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(HARNESS_PREFIX))
}

fn newest_first(left: &ChatHistoryRecord, right: &ChatHistoryRecord) -> std::cmp::Ordering {
    right
        .updated_at_unix_millis
        .cmp(&left.updated_at_unix_millis)
        .then_with(|| right.session_id.cmp(&left.session_id))
}

fn unix_millis() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis(),
        Err(_) => 0,
    }
}
