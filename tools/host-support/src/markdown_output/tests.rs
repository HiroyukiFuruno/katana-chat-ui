use katana_acp_client::{AiIntent, AiRequest, DocumentContext};
use katana_chat_ui::ChatOutputKind;

use super::{MarkdownOutputError, from_ai_response, markdown_content, unified_diff};

#[test]
fn creates_candidate_from_actual_tmp_file_context() -> Result<(), String> {
    let output = from_ai_response(
        &request(AiIntent::Create, "", "tmp/new-note.md"),
        "```markdown\n# Note\n```",
    )
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "candidate was missing".to_string())?;
    match output {
        ChatOutputKind::FileCandidate(file) => (file.path == "tmp/new-note.md"
            && file.content == "# Note")
            .then_some(())
            .ok_or_else(|| "unexpected file candidate".to_string()),
        _ => Err("expected file candidate".into()),
    }
}

#[test]
fn creates_diff_from_original_context_content() -> Result<(), String> {
    let output = from_ai_response(&request(AiIntent::Modify, "before", "tmp/note.md"), "after")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "candidate was missing".to_string())?;
    match output {
        ChatOutputKind::DiffCandidate(diff) => (diff.original_content == "before"
            && diff.updated_content == "after"
            && diff.unified_diff.contains(
                "-before\n\\ No newline at end of file\n+after\n\\ No newline at end of file",
            ))
        .then_some(())
        .ok_or_else(|| "unexpected diff candidate".to_string()),
        _ => Err("expected diff candidate".into()),
    }
}

#[test]
fn rejects_path_outside_tmp() -> Result<(), String> {
    let result = from_ai_response(&request(AiIntent::Create, "", "notes/readme.md"), "body");
    matches!(result, Err(MarkdownOutputError::UnsafePath(_)))
        .then_some(())
        .ok_or_else(|| "unsafe path was accepted".to_string())
}

#[test]
fn ignores_non_file_context() -> Result<(), String> {
    from_ai_response(&request(AiIntent::Create, "", ""), "body")
        .map_err(|error| error.to_string())?
        .is_none()
        .then_some(())
        .ok_or_else(|| "non-file context must not make a candidate".to_string())
}

#[test]
fn preserves_raw_markdown_with_an_internal_code_fence() -> Result<(), String> {
    let content = "# Title\n```rust\nlet value = 1;\n```";
    (markdown_content(content) == content)
        .then_some(())
        .ok_or_else(|| "internal code fence was unwrapped".to_string())
}

#[test]
fn unwraps_only_matching_outer_markdown_fence() -> Result<(), String> {
    let response = "````markdown\n# Title\n```rust\nlet value = 1;\n```\n````";
    (markdown_content(response) == "# Title\n```rust\nlet value = 1;\n```")
        .then_some(())
        .ok_or_else(|| "outer markdown wrapper was not unwrapped".to_string())
}

#[test]
fn preserves_unclosed_markdown_fence() -> Result<(), String> {
    let response = "```markdown\n# Incomplete";
    (markdown_content(response) == response)
        .then_some(())
        .ok_or_else(|| "unclosed fence was treated as a wrapper".to_string())
}

#[test]
fn non_markdown_file_context_does_not_make_a_candidate() -> Result<(), String> {
    from_ai_response(&request(AiIntent::Create, "", "tmp/data.txt"), "body")
        .map_err(|error| error.to_string())?
        .is_none()
        .then_some(())
        .ok_or_else(|| "non-markdown context made a candidate".to_string())
}

#[test]
fn unified_diff_preserves_empty_and_terminal_newline_state() -> Result<(), String> {
    let diff = unified_diff("tmp/note.md", "", "content");
    (diff.contains("@@ -0,0 +1,1 @@") && diff.contains("+content\n\\ No newline at end of file\n"))
        .then_some(())
        .ok_or_else(|| "empty or terminal newline state was lost".to_string())
}

fn request(intent: AiIntent, content: &str, relative_path: &str) -> AiRequest {
    let uri = if relative_path.is_empty() {
        "katanagent://chat".into()
    } else {
        format!("file://{relative_path}")
    };
    AiRequest {
        intent,
        context: DocumentContext {
            uri,
            content: content.into(),
            cursor_offset: 0,
            diagnostics: Vec::new(),
        },
        prompt: "request".into(),
        history: Vec::new(),
    }
}
