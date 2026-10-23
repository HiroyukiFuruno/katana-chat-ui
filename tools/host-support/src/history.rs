use std::{
    collections::HashSet,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatHistoryIssue {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatHistoryListing {
    pub records: Vec<ChatHistoryRecord>,
    pub issues: Vec<ChatHistoryIssue>,
}

pub(crate) struct PreparedHistorySave {
    cache_path: PathBuf,
    session_path: PathBuf,
    staged_cache: tempfile::NamedTempFile,
    staged_session: tempfile::NamedTempFile,
    old_cache: Option<Vec<u8>>,
    expected_cache: Vec<u8>,
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
        self.prepare_save(provider_id, session_id, snapshot)?
            .commit()
    }

    pub(crate) fn prepare_save(
        &self,
        provider_id: &str,
        session_id: &str,
        snapshot: ChatSessionSnapshot,
    ) -> Result<PreparedHistorySave, String> {
        validate_identifier(session_id)?;
        let record = ChatHistoryRecord::new(provider_id, session_id, snapshot);
        let root = self.provider_root(provider_id);
        let session_path = root
            .join(SESSIONS_DIRECTORY)
            .join(format!("{session_id}.json"));
        let cache_path = root.join(LAST_SESSION_FILE);
        create_parent(&session_path)?;
        let content = serde_json::to_vec_pretty(&record).map_err(|error| error.to_string())?;
        let staged_session = stage_bytes(&session_path, &content)?;
        let staged_cache = stage_bytes(&cache_path, &content)?;
        let old_cache = read_optional(&cache_path)?;
        Ok(PreparedHistorySave {
            cache_path,
            session_path,
            staged_cache,
            staged_session,
            old_cache,
            expected_cache: content,
        })
    }

    pub fn load_latest(&self) -> Result<Option<ChatHistoryRecord>, String> {
        Ok(self.list()?.into_iter().next())
    }

    pub fn load_latest_with_diagnostics(
        &self,
    ) -> Result<(Option<ChatHistoryRecord>, Vec<ChatHistoryIssue>), String> {
        let listing = self.list_with_diagnostics()?;
        Ok((listing.records.into_iter().next(), listing.issues))
    }

    pub fn load(&self, session_id: &str) -> Result<Option<ChatHistoryRecord>, String> {
        validate_identifier(session_id)?;
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.session_id == session_id))
    }

    pub fn load_with_diagnostics(
        &self,
        session_id: &str,
    ) -> Result<(Option<ChatHistoryRecord>, Vec<ChatHistoryIssue>), String> {
        validate_identifier(session_id)?;
        let listing = self.list_with_diagnostics()?;
        Ok((
            listing
                .records
                .into_iter()
                .find(|record| record.session_id == session_id),
            listing.issues,
        ))
    }

    pub fn list(&self) -> Result<Vec<ChatHistoryRecord>, String> {
        let listing = self.list_with_diagnostics()?;
        if let Some(issue) = listing.issues.first() {
            return Err(format!("{}: {}", issue.path.display(), issue.reason));
        }
        Ok(listing.records)
    }

    pub fn list_with_diagnostics(&self) -> Result<ChatHistoryListing, String> {
        let tmp = self.root.join("tmp");
        if !tmp.is_dir() {
            return Ok(ChatHistoryListing {
                records: Vec::new(),
                issues: Vec::new(),
            });
        }
        let mut listing = read_provider_records(&tmp)?;
        listing.records.sort_by(newest_first);
        let mut seen_session_ids = HashSet::new();
        listing
            .records
            .retain(|record| seen_session_ids.insert(record.session_id.clone()));
        Ok(listing)
    }

    fn provider_root(&self, provider_id: &str) -> PathBuf {
        self.root
            .join("tmp")
            .join(format!("{HARNESS_PREFIX}{}", safe_provider_id(provider_id)))
    }
}

impl PreparedHistorySave {
    pub(crate) fn commit(self) -> Result<(), String> {
        if let Err(error) = cache_matches(&self.cache_path, self.old_cache.as_deref()) {
            return Err(format!("history commit refused: {error}"));
        }
        self.staged_cache
            .persist(&self.cache_path)
            .map_err(|error| error.error.to_string())?;
        if let Err(error) = self.staged_session.persist(&self.session_path) {
            let commit_error = error.error.to_string();
            if let Err(rollback) = restore_cache(
                &self.cache_path,
                self.old_cache.as_deref(),
                &self.expected_cache,
            ) {
                return Err(format!(
                    "history commit failed: {commit_error}; cache rollback failed: {rollback}"
                ));
            }
            return Err(format!("history commit failed: {commit_error}"));
        }
        Ok(())
    }
}

fn cache_matches(path: &Path, expected: Option<&[u8]>) -> Result<(), String> {
    match fs::read(path) {
        Ok(actual) if expected.is_some_and(|expected| actual == expected) => Ok(()),
        Ok(_) => Err("cache changed before commit; refusing overwrite".to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && expected.is_none() => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err("cache changed before commit; refusing overwrite".to_string())
        }
        Err(error) => Err(format!(
            "cache check failed before commit for {}: {error}",
            path.display()
        )),
    }
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::read(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn restore_cache(
    cache_path: &Path,
    old_cache: Option<&[u8]>,
    expected_cache: &[u8],
) -> Result<(), String> {
    let current = fs::read(cache_path).map_err(|error| error.to_string())?;
    if current != expected_cache {
        return Err("cache changed after commit; refusing overwrite".to_string());
    }
    match old_cache {
        Some(content) => write_bytes(cache_path, content),
        None => fs::remove_file(cache_path).map_err(|error| error.to_string()),
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

fn stage_bytes(path: &Path, content: &[u8]) -> Result<tempfile::NamedTempFile, String> {
    let parent = path.parent().ok_or("history parent is missing")?;
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    staged
        .write_all(content)
        .map_err(|error| error.to_string())?;
    Ok(staged)
}

fn write_bytes(path: &Path, content: &[u8]) -> Result<(), String> {
    let staged = stage_bytes(path, content)?;
    staged
        .persist(path)
        .map_err(|error| error.error.to_string())?;
    Ok(())
}

fn read_provider_records(path: &Path) -> Result<ChatHistoryListing, String> {
    let mut listing = ChatHistoryListing {
        records: Vec::new(),
        issues: Vec::new(),
    };
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let provider = entry.path();
        if !is_harness_root(&provider) {
            continue;
        }
        let provider_listing = read_records(&provider.join(SESSIONS_DIRECTORY))?;
        listing.records.extend(provider_listing.records);
        listing.issues.extend(provider_listing.issues);
    }
    Ok(listing)
}

fn read_records(path: &Path) -> Result<ChatHistoryListing, String> {
    if !path.is_dir() {
        return Ok(ChatHistoryListing {
            records: Vec::new(),
            issues: Vec::new(),
        });
    }
    let mut listing = ChatHistoryListing {
        records: Vec::new(),
        issues: Vec::new(),
    };
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.path().extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        match read_record(&entry.path()) {
            Ok(record) => listing.records.push(record),
            Err(issue) => listing.issues.push(issue),
        }
    }
    Ok(listing)
}

fn read_record(path: &Path) -> Result<ChatHistoryRecord, ChatHistoryIssue> {
    let path = path.to_path_buf();
    let content = fs::read_to_string(&path).map_err(|error| ChatHistoryIssue {
        path: path.clone(),
        reason: error.to_string(),
    })?;
    let record: ChatHistoryRecord =
        serde_json::from_str(&content).map_err(|error| ChatHistoryIssue {
            path: path.clone(),
            reason: error.to_string(),
        })?;
    if record.version != HISTORY_VERSION || record.snapshot.version != 1 {
        return Err(ChatHistoryIssue {
            path,
            reason: "unsupported history version".to_string(),
        });
    }
    validate_identifier(&record.session_id).map_err(|reason| ChatHistoryIssue { path, reason })?;
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
        .then_with(|| right.provider_id.cmp(&left.provider_id))
}

fn unix_millis() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis(),
        Err(_) => 0,
    }
}
