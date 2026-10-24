use std::{fs, path::PathBuf};

use katana_chat_ui::{ChatOutput, ChatOutputKind, DiffCandidateOutput, FileCandidateOutput};

use super::ChatOutputExecutor;

#[test]
fn atomic_create_does_not_clobber_an_existing_file() -> Result<(), String> {
    let root = test_root("create");
    let path = root.join("tmp/output.md");
    fs::create_dir_all(path.parent().ok_or("missing parent")?)
        .map_err(|error| error.to_string())?;
    fs::write(&path, "original").map_err(|error| error.to_string())?;

    let result = ChatOutputExecutor::new(root.clone()).apply(&file_output("tmp/output.md", "new"));
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "original"
    );
    cleanup(root)
}

#[cfg(unix)]
#[test]
fn atomic_edit_keeps_the_existing_file_permissions() -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let root = test_root("permissions");
    let path = root.join("tmp/edited.md");
    fs::create_dir_all(path.parent().ok_or("missing parent")?)
        .map_err(|error| error.to_string())?;
    fs::write(&path, "before").map_err(|error| error.to_string())?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640))
        .map_err(|error| error.to_string())?;

    ChatOutputExecutor::new(root.clone()).apply(&diff_output())?;
    assert_eq!(
        fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "after"
    );
    assert_eq!(
        fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    cleanup(root)
}

fn file_output(path: &str, content: &str) -> ChatOutput {
    ChatOutput::new(
        1,
        1,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(path, "text/markdown", content)),
    )
}

fn diff_output() -> ChatOutput {
    ChatOutput::new(
        2,
        1,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/edited.md",
            "before",
            "after",
            "diff",
            "edit",
        )),
    )
}

fn test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kcu-host-atomic-{label}-{}",
        crate::ChatHistoryStore::new_session_id()
    ))
}

fn cleanup(path: PathBuf) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}
