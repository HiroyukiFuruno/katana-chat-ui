use super::ManualHostState;
use std::path::PathBuf;

#[test]
fn managed_host_initial_surface_is_available_without_a_ui_runtime() -> Result<(), String> {
    let state = ManualHostState::with_root(test_root())?;
    assert!(state.surface().messages.is_empty());
    assert!(state.composer_text.is_empty());
    Ok(())
}

fn test_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "kcu-manual-egui-{}",
        kcu_host_support::ChatHistoryStore::new_session_id()
    ))
}
