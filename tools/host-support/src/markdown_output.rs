//! KatanAgent の Markdown 応答を host が適用可能な候補 output へ変換する。
//!
//! `DocumentContext.uri` は workspace 相対の `file://tmp/<name>.md` を使う。

use katana_acp_client::{AiIntent, AiRequest};
use katana_chat_ui::{ChatOutputKind, DiffCandidateOutput, FileCandidateOutput};
use thiserror::Error;

const MARKDOWN_MIME_TYPE: &str = "text/markdown";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MarkdownOutputError {
    #[error("Markdown output path is outside tmp: {0}")]
    UnsafePath(String),
}

pub fn from_ai_response(
    request: &AiRequest,
    response: &str,
) -> Result<Option<ChatOutputKind>, MarkdownOutputError> {
    let Some(path) = relative_markdown_path(&request.context.uri)? else {
        return Ok(None);
    };
    let content = markdown_content(response);
    let output = match request.intent {
        AiIntent::Create => ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            path,
            MARKDOWN_MIME_TYPE,
            content,
        )),
        AiIntent::Modify => ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            &path,
            &request.context.content,
            &content,
            unified_diff(&path, &request.context.content, &content),
            format!("{path} を更新"),
        )),
        AiIntent::Autofix => return Ok(None),
    };
    Ok(Some(output))
}

fn relative_markdown_path(uri: &str) -> Result<Option<String>, MarkdownOutputError> {
    let Some(path) = uri.strip_prefix("file://") else {
        return Ok(None);
    };
    if path.starts_with('/') || has_unsafe_segment(path) {
        return Err(MarkdownOutputError::UnsafePath(path.into()));
    }
    if !markdown_extension(path) {
        return Ok(None);
    }
    if !path.starts_with("tmp/") {
        return Err(MarkdownOutputError::UnsafePath(path.into()));
    }
    Ok(Some(path.into()))
}

fn has_unsafe_segment(path: &str) -> bool {
    path.split('/').any(|part| part.is_empty() || part == "..")
}

fn markdown_extension(path: &str) -> bool {
    path.ends_with(".md") || path.ends_with(".markdown")
}

fn markdown_content(response: &str) -> String {
    outer_markdown_fence(response).unwrap_or_else(|| response.to_string())
}

fn outer_markdown_fence(response: &str) -> Option<String> {
    let (opening, body) = response.split_once('\n')?;
    let fence = markdown_fence(opening)?;
    let trimmed = body.trim_end_matches([' ', '\t', '\r', '\n']);
    let close_start = trimmed.rfind('\n').map_or(0, |index| index + 1);
    closing_fence(&trimmed[close_start..], fence)
        .then(|| body[..close_start.saturating_sub(1)].to_string())
}

fn markdown_fence(line: &str) -> Option<usize> {
    let line = line.trim();
    let fence = line
        .chars()
        .take_while(|character| *character == '`')
        .count();
    let language = line[fence..].trim();
    (fence >= 3 && matches!(language, "markdown" | "md")).then_some(fence)
}

fn closing_fence(line: &str, minimum_length: usize) -> bool {
    let line = line.trim();
    line.chars().count() >= minimum_length && line.chars().all(|character| character == '`')
}

fn unified_diff(path: &str, original: &str, updated: &str) -> String {
    format!(
        "--- a/{path}\n+++ b/{path}\n@@ -{},{} +{},{} @@\n{}{}",
        hunk_start(original),
        line_count(original),
        hunk_start(updated),
        line_count(updated),
        changed_lines('-', original),
        changed_lines('+', updated),
    )
}

fn hunk_start(content: &str) -> usize {
    if content.is_empty() { 0 } else { 1 }
}

fn changed_lines(prefix: char, content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    let mut result = content
        .split_terminator('\n')
        .map(|line| format!("{prefix}{line}\n"))
        .collect::<String>();
    if !content.ends_with('\n') {
        result.push_str("\\ No newline at end of file\n");
    }
    result
}

fn line_count(content: &str) -> usize {
    content.split_terminator('\n').count()
}

#[cfg(test)]
mod tests;
