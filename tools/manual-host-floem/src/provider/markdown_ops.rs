use katana_acp_client::{AiIntent, DocumentContext};
use katana_chat_ui::{ChatOutputKind, DiffCandidateOutput, FileCandidateOutput};

use super::{ManualProviderJob, SAMPLE_MARKDOWN_PATH};

const ATTACHED_FILE_PREFIX: &str = "Attached file: file://";
const MARKDOWN_MIME_TYPE: &str = "text/markdown";

pub(crate) enum AgentMarkdownOperation {
    Create {
        target_path: String,
    },
    Edit {
        target_path: String,
        original_content: String,
    },
}

impl AgentMarkdownOperation {
    pub(crate) fn detect(job: &ManualProviderJob) -> Option<Self> {
        if Self::is_edit_request(&job.prompt)
            && let Some(attachment) = MarkdownAttachment::extract(&job.cwd, &job.prompt)
        {
            return Some(Self::Edit {
                target_path: attachment.relative_path,
                original_content: attachment.content,
            });
        }
        AgentFileRequest::detect_target_path(&job.prompt)
            .map(|target_path| Self::Create { target_path })
    }

    pub(crate) fn intent(&self) -> AiIntent {
        match self {
            Self::Create { .. } => AiIntent::Create,
            Self::Edit { .. } => AiIntent::Modify,
        }
    }

    pub(crate) fn context(&self, cwd: &str) -> DocumentContext {
        DocumentContext {
            uri: format!("file://{cwd}/{}", self.target_path()),
            content: self.original_content().unwrap_or_default(),
            cursor_offset: 0,
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn prompt(&self, prompt: &str) -> String {
        match self {
            Self::Create { target_path } => AgentFileRequest::create_prompt(prompt, target_path),
            Self::Edit {
                target_path,
                original_content,
            } => AgentFileRequest::edit_prompt(prompt, target_path, original_content),
        }
    }

    pub(crate) fn output_kind(&self, response: &str) -> ChatOutputKind {
        let updated_content = MarkdownFileContent::extract(response);
        match self {
            Self::Create { target_path } => ChatOutputKind::FileCandidate(
                FileCandidateOutput::new(target_path, MARKDOWN_MIME_TYPE, updated_content),
            ),
            Self::Edit {
                target_path,
                original_content,
            } => ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                target_path,
                original_content,
                &updated_content,
                MarkdownDiff::unified(target_path, original_content, &updated_content),
                format!("{target_path} を更新"),
            )),
        }
    }

    fn target_path(&self) -> &str {
        match self {
            Self::Create { target_path } | Self::Edit { target_path, .. } => target_path,
        }
    }

    fn original_content(&self) -> Option<String> {
        match self {
            Self::Create { .. } => None,
            Self::Edit {
                original_content, ..
            } => Some(original_content.clone()),
        }
    }

    fn is_edit_request(prompt: &str) -> bool {
        [
            "編集", "修正", "変更", "追記", "更新", "edit", "modify", "update", "append",
        ]
        .iter()
        .any(|word| prompt.contains(word))
    }
}

pub(crate) struct AgentFileRequest;

impl AgentFileRequest {
    pub(crate) fn detect_target_path(prompt: &str) -> Option<String> {
        if prompt.contains("sample.md") && prompt.contains("tmp") {
            return Some(SAMPLE_MARKDOWN_PATH.to_string());
        }
        None
    }

    pub(crate) fn fallback_prompt(prompt: &str) -> String {
        match Self::detect_target_path(prompt) {
            Some(target_path) => Self::create_prompt(prompt, &target_path),
            None => prompt.to_string(),
        }
    }

    fn create_prompt(prompt: &str, target_path: &str) -> String {
        format!(
            "{prompt}\n\n出力対象: {target_path}\nファイル本文として使える Markdown だけを返してください。説明文やコードフェンスは付けないでください。"
        )
    }

    fn edit_prompt(prompt: &str, target_path: &str, original_content: &str) -> String {
        format!(
            "{prompt}\n\n編集対象: {target_path}\n現在の Markdown 本文:\n```markdown\n{original_content}\n```\n変更後の Markdown 本文だけを返してください。説明文やコードフェンスは付けないでください。"
        )
    }
}

pub(crate) struct MarkdownFileContent;

impl MarkdownFileContent {
    pub(crate) fn extract(content: &str) -> String {
        if let Some(fenced) = Self::first_fenced_block(content) {
            return fenced;
        }
        content.trim().to_string()
    }

    pub(crate) fn first_fenced_block(content: &str) -> Option<String> {
        let mut in_fence = false;
        let mut lines = Vec::new();
        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                if in_fence {
                    return Some(lines.join("\n"));
                }
                in_fence = true;
                continue;
            }
            if in_fence {
                lines.push(line);
            }
        }
        None
    }
}

struct MarkdownAttachment {
    relative_path: String,
    content: String,
}

impl MarkdownAttachment {
    fn extract(cwd: &str, prompt: &str) -> Option<Self> {
        let mut lines = prompt.lines();
        while let Some(line) = lines.next() {
            let Some(path) = line.trim().strip_prefix(ATTACHED_FILE_PREFIX) else {
                continue;
            };
            let remaining = lines.collect::<Vec<_>>().join("\n");
            return Self::from_path_and_content(cwd, path.trim(), &remaining);
        }
        None
    }

    fn from_path_and_content(cwd: &str, path: &str, content: &str) -> Option<Self> {
        let relative_path = Self::relative_tmp_markdown_path(cwd, path)?;
        let content = MarkdownFileContent::first_fenced_block(content)?;
        Some(Self {
            relative_path,
            content,
        })
    }

    fn relative_tmp_markdown_path(cwd: &str, absolute_path: &str) -> Option<String> {
        let prefix = format!("{}/", cwd.trim_end_matches('/'));
        let relative_path = absolute_path.strip_prefix(&prefix)?;
        if !Self::is_safe_tmp_markdown_path(relative_path) {
            return None;
        }
        Some(relative_path.to_string())
    }

    fn is_safe_tmp_markdown_path(path: &str) -> bool {
        path.starts_with("tmp/")
            && (path.ends_with(".md") || path.ends_with(".markdown"))
            && path.split('/').all(|part| !part.is_empty() && part != "..")
    }
}

pub(crate) struct MarkdownDiff;

impl MarkdownDiff {
    pub(crate) fn unified(target_path: &str, original: &str, updated: &str) -> String {
        format!(
            "--- a/{target_path}\n+++ b/{target_path}\n@@ -1,{} +1,{} @@\n{}{}",
            Self::line_count(original),
            Self::line_count(updated),
            Self::removed_lines(original),
            Self::added_lines(updated),
        )
    }

    fn removed_lines(content: &str) -> String {
        content.lines().map(|line| format!("-{line}\n")).collect()
    }

    fn added_lines(content: &str) -> String {
        content.lines().map(|line| format!("+{line}\n")).collect()
    }

    fn line_count(content: &str) -> usize {
        content.lines().count().max(1)
    }
}
