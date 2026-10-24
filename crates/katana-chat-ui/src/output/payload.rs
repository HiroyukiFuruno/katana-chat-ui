use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextOutput {
    pub text: String,
}

impl TextOutput {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeOutput {
    pub language: Option<String>,
    pub code: String,
}

impl CodeOutput {
    pub fn new(language: Option<String>, code: impl Into<String>) -> Self {
        Self {
            language,
            code: code.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileCandidateOutput {
    pub path: String,
    pub mime_type: String,
    pub content: String,
}

impl FileCandidateOutput {
    pub fn new(
        path: impl Into<String>,
        mime_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            mime_type: mime_type.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffCandidateOutput {
    pub target_path: String,
    pub original_content: String,
    pub updated_content: String,
    pub unified_diff: String,
    pub summary: String,
}

impl DiffCandidateOutput {
    pub fn new(
        target_path: impl Into<String>,
        original_content: impl Into<String>,
        updated_content: impl Into<String>,
        unified_diff: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            target_path: target_path.into(),
            original_content: original_content.into(),
            updated_content: updated_content.into(),
            unified_diff: unified_diff.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResultOutput {
    pub tool_name: String,
    pub summary: String,
}

impl ToolResultOutput {
    pub fn new(tool_name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRequestOutput {
    pub action_label: String,
    pub reason: String,
}

impl PermissionRequestOutput {
    pub fn new(action_label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            action_label: action_label.into(),
            reason: reason.into(),
        }
    }
}
