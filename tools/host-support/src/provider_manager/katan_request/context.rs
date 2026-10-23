use std::path::{Path, PathBuf};

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
    if edit_instruction(prompt)
        && let Some(context) = markdown_attachment(attachments, workspace_root)?
    {
        return Ok(context);
    }
    if create_instruction(prompt)
        && let Some(path) = markdown_path(prompt)
    {
        return create_context(&path, workspace_root);
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
        prompt.find(extension).map(|index| {
            let end = index + extension.len();
            let start = prompt[..end]
                .char_indices()
                .rev()
                .find(|(_, character)| !path_character(*character))
                .map_or(0, |(index, character)| index + character.len_utf8());
            prompt[start..end].to_string()
        })
    })
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
