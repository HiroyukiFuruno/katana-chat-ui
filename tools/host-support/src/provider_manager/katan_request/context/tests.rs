use std::{fs, path::Path};

use katana_chat_ui::{Attachment, FileResource};

use crate::markdown_output::from_ai_response;

use super::select;

#[test]
fn markdown_create_instruction_uses_tmp_document_context() -> Result<(), String> {
    let context = select(
        &[],
        "tmp/reports/status.md を作成してください",
        Path::new("/workspace"),
    )?;
    (context.document.uri == "file://tmp/reports/status.md" && context.document.content.is_empty())
        .then_some(())
        .ok_or_else(|| "create context was not built".to_string())
}

#[test]
fn japanese_instruction_without_whitespace_extracts_markdown_path() -> Result<(), String> {
    let context = select(
        &[],
        "tmp/reports/status.mdを作成してください",
        Path::new("/workspace"),
    )?;
    (context.document.uri == "file://tmp/reports/status.md")
        .then_some(())
        .ok_or_else(|| "path in Japanese instruction was not extracted".to_string())
}

#[test]
fn quoted_japanese_path_is_extracted_without_command_suffix() -> Result<(), String> {
    let context = select(&[], "「tmp/status.md」を作成", Path::new("/workspace"))?;
    (context.document.uri == "file://tmp/status.md")
        .then_some(())
        .ok_or_else(|| "quoted path was not extracted".to_string())
}

#[test]
fn markdown_attachment_requires_edit_instruction() -> Result<(), String> {
    let attachments = vec![markdown_attachment()];
    let context = select(&attachments, "この添付を要約して", Path::new("/workspace"))?;
    (context.document.uri == "katanagent://chat")
        .then_some(())
        .ok_or_else(|| "attachment-only request became an edit".to_string())
}

#[test]
fn edit_instruction_uses_typed_attachment_content() -> Result<(), String> {
    let attachments = vec![markdown_attachment()];
    let context = select(
        &attachments,
        "添付を編集してください",
        Path::new("/workspace"),
    )?;
    (context.document.uri == "file://tmp/report.md" && context.document.content == "before")
        .then_some(())
        .ok_or_else(|| "edit context was not built from attachment".to_string())
}

#[test]
fn edit_instruction_accepts_an_existing_absolute_tmp_attachment() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tmp = root.path().join("tmp");
    fs::create_dir(&tmp).map_err(|error| error.to_string())?;
    let source = tmp.join("report.md");
    fs::write(&source, "before").map_err(|error| error.to_string())?;
    let attachment = Attachment::file(FileResource::new(
        format!("file://{}", source.display()),
        "text/markdown",
        "before",
        6,
    ));

    let context = select(&[attachment], "添付を編集してください", root.path())?;
    (context.document.uri == "file://tmp/report.md" && context.document.content == "before")
        .then_some(())
        .ok_or_else(|| "existing absolute attachment was not accepted".to_string())
}

#[test]
fn create_instruction_accepts_a_new_absolute_tmp_path() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tmp = root.path().join("tmp");
    fs::create_dir(&tmp).map_err(|error| error.to_string())?;
    let target = tmp.join("new.md");
    let context = select(&[], &format!("{} を作成", target.display()), root.path())?;
    assert_eq!(context.document.uri, "file://tmp/new.md");
    assert!(!target.exists());
    Ok(())
}

#[test]
fn create_instruction_rejects_a_new_absolute_path_outside_tmp() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let target = root.path().join("new.md");
    assert!(select(&[], &format!("{} を作成", target.display()), root.path()).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn create_instruction_rejects_an_absolute_symlink_escape() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let outside = tempfile::tempdir().map_err(|error| error.to_string())?;
    std::os::unix::fs::symlink(outside.path(), root.path().join("tmp"))
        .map_err(|error| error.to_string())?;
    let target = root.path().join("tmp/new.md");
    assert!(select(&[], &format!("{} を作成", target.display()), root.path()).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn create_instruction_rejects_a_dangling_absolute_symlink() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tmp = root.path().join("tmp");
    fs::create_dir(&tmp).map_err(|error| error.to_string())?;
    let target = tmp.join("new.md");
    std::os::unix::fs::symlink(root.path().join("missing.md"), &target)
        .map_err(|error| error.to_string())?;
    assert!(select(&[], &format!("{} を作成", target.display()), root.path()).is_err());
    Ok(())
}

#[test]
fn create_instruction_rejects_markdown_path_outside_tmp() -> Result<(), String> {
    select(&[], "notes/report.md を作成", Path::new("/workspace"))
        .is_err()
        .then_some(())
        .ok_or_else(|| "unsafe markdown path was accepted".to_string())
}

#[test]
fn create_instruction_normalizes_current_directory_components() -> Result<(), String> {
    for path in ["./tmp/report.md", "tmp/./report.md", "./tmp/./report.md"] {
        let context = select(&[], &format!("{path} を作成"), Path::new("/workspace"))?;
        assert_eq!(context.document.uri, "file://tmp/report.md");
    }
    Ok(())
}

#[test]
fn current_directory_normalization_does_not_accept_parent_traversal() {
    for path in [
        "./tmp/../report.md",
        "tmp/../tmp/report.md",
        "./notes/report.md",
    ] {
        assert!(select(&[], &format!("{path} を作成"), Path::new("/workspace")).is_err());
    }
}

#[test]
fn non_markdown_attachment_keeps_chat_context() -> Result<(), String> {
    let attachments = vec![Attachment::file(FileResource::new(
        "file://tmp/data.txt",
        "text/plain",
        "payload",
        7,
    ))];
    let context = select(&attachments, "添付を編集", Path::new("/workspace"))?;
    (context.document.uri == "katanagent://chat")
        .then_some(())
        .ok_or_else(|| "non-markdown attachment was discarded into an edit context".to_string())
}

#[test]
fn create_context_flows_into_a_file_candidate() -> Result<(), String> {
    let context = select(&[], "tmp/generated.md を作成", Path::new("/workspace"))?;
    let request = katana_acp_client::AiRequest {
        intent: context.intent,
        context: context.document,
        prompt: "tmp/generated.md を作成".into(),
        history: Vec::new(),
    };
    let output = from_ai_response(&request, "# Generated")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "structured output was missing".to_string())?;
    matches!(output, katana_chat_ui::ChatOutputKind::FileCandidate(file) if file.path == "tmp/generated.md")
        .then_some(())
        .ok_or_else(|| "request context did not produce a file candidate".to_string())
}

fn markdown_attachment() -> Attachment {
    Attachment::file(FileResource::new(
        "file://tmp/report.md",
        "text/markdown",
        "before",
        6,
    ))
}
