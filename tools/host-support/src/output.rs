mod path;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod atomic_tests;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use katana_chat_ui::{
    ChatOutput, ChatOutputKind, DiffCandidateOutput, FileCandidateOutput, OutputStatus,
};

#[derive(Clone)]
pub struct ChatOutputExecutor {
    root: PathBuf,
}

impl ChatOutputExecutor {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn apply(&self, output: &ChatOutput) -> Result<PathBuf, String> {
        match &output.kind {
            ChatOutputKind::FileCandidate(file) => self.create_file(file),
            ChatOutputKind::DiffCandidate(diff) => self.apply_diff(diff),
            _ => Err("output is not a mutable file candidate".to_string()),
        }
    }

    pub fn undo(&self, output: &ChatOutput) -> Result<PathBuf, String> {
        if output.status != OutputStatus::Applied {
            return Err(format!("output is not applied: {}", output.id));
        }
        match &output.kind {
            ChatOutputKind::FileCandidate(file) => self.undo_file(file),
            ChatOutputKind::DiffCandidate(diff) => self.undo_diff(diff),
            _ => Err("output is not undoable".to_string()),
        }
    }

    fn create_file(&self, file: &FileCandidateOutput) -> Result<PathBuf, String> {
        let path = self.target_path(&file.path, true)?;
        create_new(&path, &file.content)?;
        Ok(path)
    }

    fn apply_diff(&self, diff: &DiffCandidateOutput) -> Result<PathBuf, String> {
        let path = self.target_path(&diff.target_path, false)?;
        require_content(&path, &diff.original_content)?;
        replace_content(&path, &diff.updated_content)?;
        Ok(path)
    }

    fn undo_file(&self, file: &FileCandidateOutput) -> Result<PathBuf, String> {
        let path = self.target_path(&file.path, false)?;
        require_content(&path, &file.content)?;
        fs::remove_file(&path).map_err(|error| error.to_string())?;
        Ok(path)
    }

    fn undo_diff(&self, diff: &DiffCandidateOutput) -> Result<PathBuf, String> {
        let path = self.target_path(&diff.target_path, false)?;
        require_content(&path, &diff.updated_content)?;
        replace_content(&path, &diff.original_content)?;
        Ok(path)
    }

    fn target_path(&self, requested: &str, create_parents: bool) -> Result<PathBuf, String> {
        path::target_path(&self.root, requested, create_parents)
    }
}

fn create_new(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("output parent is missing")?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|error| error.to_string())?;
    file.persist_noclobber(path)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn require_content(path: &Path, expected: &str) -> Result<(), String> {
    let actual = fs::read_to_string(path).map_err(|error| error.to_string())?;
    (actual == expected)
        .then_some(())
        .ok_or_else(|| format!("output content changed: {}", path.display()))
}

fn replace_content(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("output parent is missing")?;
    let permissions = fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|error| error.to_string())?;
    file.as_file()
        .set_permissions(permissions)
        .map_err(|error| error.to_string())?;
    file.persist(path).map_err(|error| error.to_string())?;
    Ok(())
}
