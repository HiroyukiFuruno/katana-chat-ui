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

#[test]
fn malformed_sibling_does_not_hide_valid_history() -> Result<(), String> {
    let root = test_root("isolated-json");
    let store = ChatHistoryStore::new(root.clone());
    store.save("katanagent", "session-valid", ChatSession::new().snapshot())?;
    let bad = root.join("tmp/harness-katanagent/sessions/bad.json");
    fs::write(&bad, "not json").map_err(|error| error.to_string())?;

    let listing = store.list_with_diagnostics()?;
    assert_eq!(listing.records.len(), 1);
    assert_eq!(listing.issues.len(), 1);
    assert_eq!(listing.issues[0].path, bad);
    assert!(store.load_latest_with_diagnostics()?.0.is_some());
    assert!(bad.is_file());
    cleanup(root)
}

#[test]
fn unsupported_version_is_isolated_from_valid_history() -> Result<(), String> {
    let root = test_root("isolated-version");
    let store = ChatHistoryStore::new(root.clone());
    store.save("katanagent", "session-valid", ChatSession::new().snapshot())?;
    let bad = root.join("tmp/harness-katanagent/sessions/old.json");
    let record = ChatHistoryRecord {
        version: 99,
        session_id: "session-old".to_string(),
        provider_id: "katanagent".to_string(),
        updated_at_unix_millis: 1,
        snapshot: ChatSession::new().snapshot(),
    };
    fs::write(
        &bad,
        serde_json::to_string(&record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let listing = store.list_with_diagnostics()?;
    assert_eq!(listing.records.len(), 1);
    assert_eq!(listing.issues[0].path, bad);
    assert!(
        listing.issues[0]
            .reason
            .contains("unsupported history version")
    );
    cleanup(root)
}

#[test]
fn duplicate_session_ids_keep_newest_record_and_diagnostics() -> Result<(), String> {
    let root = test_root("duplicate-session-id");
    let store = ChatHistoryStore::new(root.clone());
    let broken_path = root.join("tmp/harness-copilot/sessions/broken.json");
    write_duplicate_records(&root)?;
    fs::write(&broken_path, "not json").map_err(|error| error.to_string())?;

    let listing = store.list_with_diagnostics()?;
    assert_listing(&listing, &broken_path)?;
    assert!(broken_path.is_file());

    assert_loaded_shared_session(&store)?;
    cleanup(root)
}

#[test]
fn unreadable_record_is_isolated_without_deletion() -> Result<(), String> {
    let root = test_root("isolated-read");
    let store = ChatHistoryStore::new(root.clone());
    store.save("katanagent", "session-valid", ChatSession::new().snapshot())?;
    let bad = root.join("tmp/harness-katanagent/sessions/read-error.json");
    fs::create_dir(&bad).map_err(|error| error.to_string())?;

    let listing = store.list_with_diagnostics()?;
    assert_eq!(listing.records.len(), 1);
    assert_eq!(listing.issues[0].path, bad);
    assert!(bad.is_dir());
    cleanup(root)
}

#[test]
fn prepared_save_restores_cache_when_session_commit_is_blocked() -> Result<(), String> {
    let root = test_root("prepared-rollback");
    let store = ChatHistoryStore::new(root.clone());
    let mut first = ChatSession::new();
    first.draft_mut().set_text("first snapshot");
    store.save("katanagent", "session-1", first.snapshot())?;
    let cache = root.join("tmp/harness-katanagent/last-session.json");
    let before = fs::read(&cache).map_err(|error| error.to_string())?;
    let session_path = root.join("tmp/harness-katanagent/sessions/session-1.json");
    fs::remove_file(&session_path).map_err(|error| error.to_string())?;
    fs::create_dir(&session_path).map_err(|error| error.to_string())?;

    let mut second = ChatSession::new();
    second.draft_mut().set_text("second snapshot");
    let result = store
        .prepare_save("katanagent", "session-1", second.snapshot())?
        .commit();

    assert!(result.is_err());
    assert_eq!(fs::read(&cache).map_err(|error| error.to_string())?, before);
    assert!(session_path.is_dir());
    cleanup(root)
}

#[test]
fn cache_rollback_refuses_external_change() -> Result<(), String> {
    let root = test_root("cache-race");
    let cache = root.join("tmp/cache.json");
    fs::create_dir_all(cache.parent().ok_or("missing parent")?)
        .map_err(|error| error.to_string())?;
    fs::write(&cache, "external").map_err(|error| error.to_string())?;

    let result = super::restore_cache(&cache, None, b"prepared");

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(&cache).map_err(|error| error.to_string())?,
        "external"
    );
    cleanup(root)
}

#[test]
fn cache_rollback_removes_cache_when_none_existed() -> Result<(), String> {
    let root = test_root("cache-absent");
    let cache = root.join("tmp/cache.json");
    fs::create_dir_all(cache.parent().ok_or("missing parent")?)
        .map_err(|error| error.to_string())?;
    fs::write(&cache, "prepared").map_err(|error| error.to_string())?;

    super::restore_cache(&cache, None, b"prepared")?;

    assert!(!cache.exists());
    cleanup(root)
}

#[test]
fn prepared_save_refuses_cache_changed_before_commit() -> Result<(), String> {
    let root = test_root("cache-before-commit");
    let store = ChatHistoryStore::new(root.clone());
    let mut first = ChatSession::new();
    first.draft_mut().set_text("first");
    store.save("katanagent", "session-1", first.snapshot())?;
    let cache = root.join("tmp/harness-katanagent/last-session.json");
    let session = root.join("tmp/harness-katanagent/sessions/session-1.json");
    let before_session = fs::read(&session).map_err(|error| error.to_string())?;
    let mut second = ChatSession::new();
    second.draft_mut().set_text("second");
    let prepared = store.prepare_save("katanagent", "session-1", second.snapshot())?;
    fs::write(&cache, "external").map_err(|error| error.to_string())?;

    assert!(prepared.commit().is_err());
    assert_eq!(
        fs::read(&cache).map_err(|error| error.to_string())?,
        b"external"
    );
    assert_eq!(
        fs::read(&session).map_err(|error| error.to_string())?,
        before_session
    );
    cleanup(root)
}

#[test]
fn prepared_save_refuses_cache_created_after_absent_prepare() -> Result<(), String> {
    let root = test_root("cache-created-before-commit");
    let store = ChatHistoryStore::new(root.clone());
    let cache = root.join("tmp/harness-katanagent/last-session.json");
    let mut session = ChatSession::new();
    session.draft_mut().set_text("prepared");
    let prepared = store.prepare_save("katanagent", "session-1", session.snapshot())?;
    fs::write(&cache, "external").map_err(|error| error.to_string())?;

    assert!(prepared.commit().is_err());
    assert_eq!(
        fs::read(&cache).map_err(|error| error.to_string())?,
        b"external"
    );
    cleanup(root)
}

fn write_duplicate_records(root: &std::path::Path) -> Result<(), String> {
    write_record(
        &history_path(root, "claude-code", "session-shared"),
        record(
            "claude-code",
            "session-shared",
            10,
            "old draft",
            "old-model",
        ),
    )?;
    write_record(
        &history_path(root, "copilot", "session-shared"),
        record("copilot", "session-shared", 20, "new draft", "new-model"),
    )?;
    write_record(
        &history_path(root, "claude-code", "session-other"),
        record(
            "claude-code",
            "session-other",
            15,
            "other draft",
            "other-model",
        ),
    )?;
    write_tied_records(root)
}

fn write_tied_records(root: &std::path::Path) -> Result<(), String> {
    write_record(
        &history_path(root, "claude-code", "session-tie"),
        record("claude-code", "session-tie", 30, "tie left", "left-model"),
    )?;
    write_record(
        &history_path(root, "copilot", "session-tie"),
        record("copilot", "session-tie", 30, "tie right", "right-model"),
    )
}

fn record(
    provider_id: &str,
    session_id: &str,
    updated_at_unix_millis: u128,
    draft: &str,
    model: &str,
) -> ChatHistoryRecord {
    let mut session = ChatSession::new();
    session.draft_mut().set_text(draft);
    let mut snapshot = session.snapshot();
    snapshot.vendor_state.selected_model = Some(model.to_string());
    ChatHistoryRecord {
        version: 1,
        session_id: session_id.to_string(),
        provider_id: provider_id.to_string(),
        updated_at_unix_millis,
        snapshot,
    }
}

fn history_path(root: &std::path::Path, provider_id: &str, session_id: &str) -> PathBuf {
    root.join(format!(
        "tmp/harness-{provider_id}/sessions/{session_id}.json"
    ))
}

fn assert_listing(
    listing: &super::ChatHistoryListing,
    broken_path: &std::path::Path,
) -> Result<(), String> {
    assert_eq!(listing.records.len(), 3);
    assert_eq!(listing.issues.len(), 1);
    assert_eq!(listing.issues[0].path, broken_path);
    let shared: Vec<_> = listing
        .records
        .iter()
        .filter(|record| record.session_id == "session-shared")
        .collect();
    assert_eq!(shared.len(), 1);
    assert_eq!(shared[0].provider_id, "copilot");
    assert_eq!(shared[0].snapshot.draft.text, "new draft");
    assert_eq!(
        shared[0].snapshot.vendor_state.selected_model.as_deref(),
        Some("new-model")
    );
    let tie = listing
        .records
        .iter()
        .find(|record| record.session_id == "session-tie")
        .ok_or("tied session is missing")?;
    assert_eq!(tie.provider_id, "copilot");
    Ok(())
}

fn assert_loaded_shared_session(store: &ChatHistoryStore) -> Result<(), String> {
    let (loaded, issues) = store.load_with_diagnostics("session-shared")?;
    let loaded = loaded.ok_or("deduplicated session did not load")?;
    assert_eq!(loaded.provider_id, "copilot");
    assert_eq!(loaded.snapshot.draft.text, "new draft");
    assert_eq!(
        loaded.snapshot.vendor_state.selected_model.as_deref(),
        Some("new-model")
    );
    assert_eq!(issues.len(), 1);
    Ok(())
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

fn write_record(path: &std::path::Path, record: ChatHistoryRecord) -> Result<(), String> {
    let parent = path.parent().ok_or("missing record parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    fs::write(
        path,
        serde_json::to_string(&record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}
