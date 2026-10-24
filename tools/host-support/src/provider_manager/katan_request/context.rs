use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use katana_acp_client::{AiIntent, DocumentContext};
use katana_chat_ui::Attachment;

pub(super) struct RequestContext {
    pub(super) intent: AiIntent,
    pub(super) document: DocumentContext,
}

pub(super) fn select(
    attachments: &[Attachment],
    prompt: &str,
    workspace_root: &Path,
) -> Result<RequestContext, String> {
    let editing = edit_instruction(prompt);
    if editing && let Some(context) = markdown_attachment(attachments, workspace_root)? {
        return Ok(context);
    }
    if create_instruction(prompt)
        && let Some(path) = markdown_path(prompt)
    {
        return create_context(&path, workspace_root);
    }
    if editing && let Some(context) = edit_context(prompt, workspace_root)? {
        return Ok(context);
    }
    Ok(chat_context())
}

fn chat_context() -> RequestContext {
    RequestContext {
        intent: AiIntent::Create,
        document: DocumentContext {
            uri: "katanagent://chat".into(),
            content: String::new(),
            cursor_offset: 0,
            diagnostics: Vec::new(),
        },
    }
}

fn markdown_attachment(
    attachments: &[Attachment],
    workspace_root: &Path,
) -> Result<Option<RequestContext>, String> {
    for attachment in attachments {
        let Attachment::FileResource(file) = attachment else {
            continue;
        };
        let Some(path) = markdown_file_uri(&file.uri, workspace_root)? else {
            continue;
        };
        return Ok(Some(RequestContext {
            intent: AiIntent::Modify,
            document: document(path, file.text.clone()),
        }));
    }
    Ok(None)
}

fn create_context(path: &str, workspace_root: &Path) -> Result<RequestContext, String> {
    let path = relative_tmp_path(path, workspace_root, true)?;
    Ok(RequestContext {
        intent: AiIntent::Create,
        document: document(path, String::new()),
    })
}

fn edit_context(prompt: &str, workspace_root: &Path) -> Result<Option<RequestContext>, String> {
    let mut targets = Vec::new();
    for candidate in markdown_paths(prompt) {
        let target = resolve_edit_target(&candidate, workspace_root)?;
        if !targets.contains(&target) {
            targets.push(target);
        }
    }
    if targets.len() > 1 {
        return Err("Specify one Markdown edit target".into());
    }
    let Some((path, target)) = targets.pop() else {
        return Ok(None);
    };
    let content = fs::read_to_string(&target)
        .map_err(|error| format!("Markdown edit target could not be read: {error}"))?;
    Ok(Some(RequestContext {
        intent: AiIntent::Modify,
        document: document(path, content),
    }))
}

fn resolve_edit_target(path: &str, workspace_root: &Path) -> Result<(String, PathBuf), String> {
    if Path::new(path)
        .components()
        .any(|part| part == Component::ParentDir)
    {
        return Err("Markdown edit target contains parent traversal".into());
    }
    let relative = relative_tmp_path(path, workspace_root, false)?;
    let root = workspace_root
        .canonicalize()
        .map_err(|error| format!("workspace root is unavailable: {error}"))?;
    let target = root
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("Markdown edit target is unavailable: {error}"))?;
    let relative = edit_relative_path(&target, &root)?;
    if !target
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("Markdown edit target is not a file".into());
    }
    Ok((relative, target))
}

fn edit_relative_path(target: &Path, root: &Path) -> Result<String, String> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| "Markdown edit target is outside workspace".to_string())?
        .to_str()
        .ok_or("Markdown edit target is not a UTF-8 path")?
        .to_string();
    if !valid_tmp_markdown_path(&relative) {
        return Err("Markdown edit target is outside tmp".into());
    }
    Ok(relative)
}

fn document(path: String, content: String) -> DocumentContext {
    DocumentContext {
        uri: format!("file://{path}"),
        content,
        cursor_offset: 0,
        diagnostics: Vec::new(),
    }
}

fn markdown_file_uri(uri: &str, workspace_root: &Path) -> Result<Option<String>, String> {
    let Some(path) = uri.strip_prefix("file://") else {
        return Ok(None);
    };
    if !is_markdown_path(path) {
        return Ok(None);
    }
    relative_tmp_path(path, workspace_root, false).map(Some)
}

fn relative_tmp_path(path: &str, workspace_root: &Path, creating: bool) -> Result<String, String> {
    let path = if path.starts_with('/') {
        absolute_relative_path(path, workspace_root, creating)?
    } else {
        path.to_string()
    };
    let path = path
        .split('/')
        .filter(|component| *component != ".")
        .collect::<Vec<_>>()
        .join("/");
    if valid_tmp_markdown_path(&path) {
        Ok(path)
    } else {
        Err(format!("Markdown output path is outside tmp: {path}"))
    }
}

fn absolute_relative_path(
    path: &str,
    workspace_root: &Path,
    creating: bool,
) -> Result<String, String> {
    let root = workspace_root
        .canonicalize()
        .map_err(|error| format!("workspace root is unavailable: {error}"))?;
    resolve_absolute_path(Path::new(path), creating)
        .map_err(|error| format!("Markdown output path is unavailable: {error}"))?
        .strip_prefix(root)
        .map_err(|_| format!("Markdown output path is outside workspace: {path}"))
        .map(|relative| relative.to_string_lossy().into_owned())
}

fn resolve_absolute_path(path: &Path, creating: bool) -> std::io::Result<PathBuf> {
    match path.canonicalize() {
        Ok(resolved) => Ok(resolved),
        Err(error) if creating && error.kind() == std::io::ErrorKind::NotFound => {
            match path.symlink_metadata() {
                Err(missing) if missing.kind() == std::io::ErrorKind::NotFound => {
                    let parent = path.parent().ok_or(error)?;
                    let name = path.file_name().ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing file name")
                    })?;
                    Ok(parent.canonicalize()?.join(name))
                }
                _ => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

fn valid_tmp_markdown_path(path: &str) -> bool {
    path.starts_with("tmp/")
        && is_markdown_path(path)
        && path.split('/').all(|part| !part.is_empty() && part != "..")
}

fn is_markdown_path(path: &str) -> bool {
    path.ends_with(".md") || path.ends_with(".markdown")
}

fn markdown_path(prompt: &str) -> Option<String> {
    [".markdown", ".md"].into_iter().find_map(|extension| {
        prompt
            .find(extension)
            .map(|index| path_ending_at(prompt, index + extension.len()))
    })
}

fn markdown_paths(prompt: &str) -> Vec<String> {
    [".markdown", ".md"]
        .into_iter()
        .flat_map(|extension| {
            prompt
                .match_indices(extension)
                .map(move |(index, _)| path_ending_at(prompt, index + extension.len()))
        })
        .collect()
}

fn path_ending_at(prompt: &str, end: usize) -> String {
    let start = prompt[..end]
        .char_indices()
        .rev()
        .find(|(_, character)| !path_character(*character))
        .map_or(0, |(index, character)| index + character.len_utf8());
    prompt[start..end].to_string()
}

fn path_character(character: char) -> bool {
    !character.is_whitespace() && !"`'\"()[]{},:;!?「」".contains(character)
}

fn create_instruction(prompt: &str) -> bool {
    markdown_path(prompt).is_some()
        && contains_any(prompt, &["作成", "生成", "create", "generate", "write"])
}

fn edit_instruction(prompt: &str) -> bool {
    contains_any(
        prompt,
        &["編集", "修正", "変更", "更新", "edit", "modify", "update"],
    )
}

fn contains_any(prompt: &str, words: &[&str]) -> bool {
    let prompt = prompt.to_lowercase();
    words.iter().any(|word| prompt.contains(word))
}

#[cfg(test)]
mod tests;
