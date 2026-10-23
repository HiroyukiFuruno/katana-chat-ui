use std::{fs, path::PathBuf};

use katana_chat_ui::{
    ChatOutput, ChatOutputKind, DiffCandidateOutput, FileCandidateOutput, OutputStatus,
};

use super::ChatOutputExecutor;

#[test]
fn create_new_and_undo_preserve_existing_or_changed_files() -> Result<(), String> {
    let root = test_root("create");
    fs::create_dir_all(root.join("tmp")).map_err(|error| error.to_string())?;
    let existing = root.join("tmp/existing.md");
    fs::write(&existing, "keep").map_err(|error| error.to_string())?;
    let executor = ChatOutputExecutor::new(root.clone());
    assert!(
        executor
            .apply(&file_output(1, "tmp/existing.md", "new"))
            .is_err()
    );
    assert_eq!(
        fs::read_to_string(&existing).map_err(|error| error.to_string())?,
        "keep"
    );

    let output = file_output(2, "tmp/generated.md", "generated");
    executor.apply(&output)?;
    assert!(executor.undo(&output).is_err());
    let applied = applied(output);
    fs::write(root.join("tmp/generated.md"), "changed").map_err(|error| error.to_string())?;
    assert!(executor.undo(&applied).is_err());
    assert_eq!(
        fs::read_to_string(root.join("tmp/generated.md")).map_err(|error| error.to_string())?,
        "changed"
    );
    cleanup(root)
}

#[test]
fn diff_requires_original_content_and_restores_only_its_applied_edit() -> Result<(), String> {
    let root = test_root("diff");
    let path = root.join("tmp/edited.md");
    fs::create_dir_all(path.parent().ok_or("missing parent")?)
        .map_err(|error| error.to_string())?;
    fs::write(&path, "other").map_err(|error| error.to_string())?;
    let executor = ChatOutputExecutor::new(root.clone());
    let output = diff_output(1);
    assert!(executor.apply(&output).is_err());
    assert_eq!(
        fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "other"
    );

    fs::write(&path, "before").map_err(|error| error.to_string())?;
    executor.apply(&output)?;
    fs::write(&path, "changed").map_err(|error| error.to_string())?;
    assert!(executor.undo(&applied(output.clone())).is_err());
    assert_eq!(
        fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "changed"
    );
    cleanup(root)
}

#[cfg(unix)]
#[test]
fn symlink_under_tmp_cannot_escape_the_output_root() -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let root = test_root("symlink-root");
    let outside = test_root("symlink-outside");
    fs::create_dir_all(root.join("tmp")).map_err(|error| error.to_string())?;
    fs::create_dir_all(&outside).map_err(|error| error.to_string())?;
    symlink(&outside, root.join("tmp/escape")).map_err(|error| error.to_string())?;
    let executor = ChatOutputExecutor::new(root.clone());
    assert!(
        executor
            .apply(&file_output(1, "tmp/escape/owned.md", "owned"))
            .is_err()
    );
    assert!(!outside.join("owned.md").exists());
    cleanup(root)?;
    cleanup(outside)
}

fn file_output(id: u64, path: &str, content: &str) -> ChatOutput {
    ChatOutput::new(
        id,
        1,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(path, "text/markdown", content)),
    )
}

fn diff_output(id: u64) -> ChatOutput {
    ChatOutput::new(
        id,
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

fn applied(mut output: ChatOutput) -> ChatOutput {
    output.status = OutputStatus::Applied;
    output
}

fn test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kcu-host-output-{label}-{}",
        crate::ChatHistoryStore::new_session_id()
    ))
}

fn cleanup(path: PathBuf) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}
