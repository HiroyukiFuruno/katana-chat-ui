use std::{fs, path::Path};

use katana_acp_client::AiIntent;
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
fn create_instruction_keeps_precedence_when_the_file_name_contains_edit() -> Result<(), String> {
    let context = select(
        &[],
        "tmp/edit-notes.md を作成してください",
        Path::new("/workspace"),
    )?;
    assert_eq!(context.intent, AiIntent::Create);
    assert_eq!(context.document.uri, "file://tmp/edit-notes.md");
    assert!(context.document.content.is_empty());
    Ok(())
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

#[test]
fn explicit_existing_markdown_edit_reads_workspace_file_and_builds_diff() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    for relative in ["tmp/report.md", "tmp/report.markdown"] {
        assert_existing_edit_context(root.path(), relative)?;
    }
    Ok(())
}

fn assert_existing_edit_context(root: &Path, relative: &str) -> Result<(), String> {
    let source = root.join(relative);
    fs::write(&source, "original file content").map_err(|error| error.to_string())?;
    for target in [
        relative.to_string(),
        format!("./{relative}"),
        source.display().to_string(),
    ] {
        assert_edit_target(root, &target, relative)?;
    }
    Ok(())
}

fn assert_edit_target(root: &Path, target: &str, relative: &str) -> Result<(), String> {
    let context = select(&[], &format!("{target} を編集してください"), root)?;
    if context.intent != AiIntent::Modify
        || context.document.uri != format!("file://{relative}")
        || context.document.content != "original file content"
    {
        return Err(format!("filesystem edit context mismatch for {relative}"));
    }
    let request = katana_acp_client::AiRequest {
        intent: context.intent,
        context: context.document,
        prompt: relative.into(),
        history: Vec::new(),
    };
    assert_diff_candidate(&request, relative)
}

fn assert_diff_candidate(
    request: &katana_acp_client::AiRequest,
    relative: &str,
) -> Result<(), String> {
    let output = from_ai_response(request, "updated file content")
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("diff candidate missing for {relative}"))?;
    match output {
        katana_chat_ui::ChatOutputKind::DiffCandidate(diff)
            if diff.target_path == relative
                && diff.original_content == "original file content"
                && diff.updated_content == "updated file content" =>
        {
            Ok(())
        }
        _ => Err(format!("filesystem edit diff mismatch for {relative}")),
    }
}

#[test]
fn explicit_markdown_edit_rejects_missing_and_directory_targets() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    let directory = root.path().join("tmp/directory.md");
    fs::create_dir(&directory).map_err(|error| error.to_string())?;
    for prompt in [
        "tmp/missing.md を編集してください".to_string(),
        "tmp/directory.md を編集してください".to_string(),
        format!(
            "{} を編集してください",
            root.path().join("tmp/missing.markdown").display()
        ),
        format!("{} を編集してください", directory.display()),
    ] {
        if select(&[], &prompt, root.path()).is_ok() {
            return Err(format!(
                "unsafe or unavailable edit target was accepted: {prompt}"
            ));
        }
    }
    Ok(())
}

#[test]
fn explicit_edit_rejects_ambiguous_markdown_targets_without_attachments() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tmp = root.path().join("tmp");
    fs::create_dir(&tmp).map_err(|error| error.to_string())?;
    fs::write(tmp.join("first.md"), "first").map_err(|error| error.to_string())?;
    fs::write(tmp.join("second.md"), "second").map_err(|error| error.to_string())?;
    let prompt = "tmp/first.md と tmp/second.md を編集してください";
    select(&[], prompt, root.path())
        .is_err()
        .then_some(())
        .ok_or_else(|| "multiple explicit markdown targets were accepted".to_string())
}

#[test]
fn repeated_edit_path_and_current_directory_alias_use_one_target() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    fs::write(root.path().join("tmp/report.md"), "before").map_err(|error| error.to_string())?;
    let context = select(
        &[],
        "tmp/report.md を編集し ./tmp/report.md を更新",
        root.path(),
    )?;
    assert_eq!(context.intent, AiIntent::Modify);
    assert_eq!(context.document.uri, "file://tmp/report.md");
    assert_eq!(context.document.content, "before");
    Ok(())
}

#[cfg(unix)]
#[test]
fn edit_symlink_cannot_read_a_workspace_file_outside_tmp() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    fs::write(root.path().join("private.md"), "outside allowed tmp")
        .map_err(|error| error.to_string())?;
    std::os::unix::fs::symlink(
        root.path().join("private.md"),
        root.path().join("tmp/alias.md"),
    )
    .map_err(|error| error.to_string())?;
    assert!(select(&[], "tmp/alias.md を編集", root.path()).is_err());
    Ok(())
}

#[test]
fn explicit_markdown_edit_rejects_invalid_utf8_before_chat_fallback() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    let relative = root.path().join("tmp/invalid.md");
    fs::write(&relative, [0xff, 0xfe, 0xfd]).map_err(|error| error.to_string())?;
    for prompt in [
        "tmp/invalid.md を編集してください".to_string(),
        format!("{} を編集してください", relative.display()),
    ] {
        if select(&[], &prompt, root.path()).is_ok() {
            return Err(format!("invalid UTF-8 target was accepted: {prompt}"));
        }
    }
    Ok(())
}

#[test]
fn explicit_markdown_edit_rejects_outside_and_parent_traversal_targets() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    let outside = tempfile::tempdir().map_err(|error| error.to_string())?;
    let outside_file = outside.path().join("outside.md");
    fs::write(&outside_file, "outside").map_err(|error| error.to_string())?;
    for prompt in [
        "notes/report.md を編集してください".to_string(),
        "tmp/../report.md を編集してください".to_string(),
        format!("{} を編集してください", outside_file.display()),
        format!(
            "{} を編集してください",
            root.path().join("tmp/../report.md").display()
        ),
    ] {
        if select(&[], &prompt, root.path()).is_ok() {
            return Err(format!(
                "outside or traversal target was accepted: {prompt}"
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn explicit_markdown_edit_rejects_symlink_escape_before_reading_data() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let outside = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::write(outside.path().join("escape.md"), "outside").map_err(|error| error.to_string())?;
    fs::create_dir(root.path().join("tmp")).map_err(|error| error.to_string())?;
    std::os::unix::fs::symlink(
        outside.path().join("escape.md"),
        root.path().join("tmp/escape.md"),
    )
    .map_err(|error| error.to_string())?;
    let prompt = "tmp/escape.md を編集してください";
    if select(&[], prompt, root.path()).is_ok() {
        return Err("relative symlink escape was accepted".into());
    }
    let absolute = format!(
        "{} を編集してください",
        root.path().join("tmp/escape.md").display()
    );
    if select(&[], &absolute, root.path()).is_ok() {
        return Err("absolute symlink escape was accepted".into());
    }
    Ok(())
}

#[test]
fn markdown_attachment_precedes_prompt_path_without_filesystem_read() -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tmp = root.path().join("tmp");
    fs::create_dir(&tmp).map_err(|error| error.to_string())?;
    fs::write(tmp.join("prompt.md"), "prompt file must not win")
        .map_err(|error| error.to_string())?;
    let attachment = markdown_attachment();
    for prompt in [
        "tmp/prompt.md を編集してください",
        "tmp/missing-prompt.md を編集してください",
    ] {
        let context = select(std::slice::from_ref(&attachment), prompt, root.path())?;
        if context.intent != AiIntent::Modify
            || context.document.uri != "file://tmp/report.md"
            || context.document.content != "before"
        {
            return Err(format!("legacy attachment precedence changed for {prompt}"));
        }
    }
    Ok(())
}

fn markdown_attachment() -> Attachment {
    Attachment::file(FileResource::new(
        "file://tmp/report.md",
        "text/markdown",
        "before",
        6,
    ))
}
