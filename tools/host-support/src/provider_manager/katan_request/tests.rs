use katana_chat_ui::{Attachment, ChatSession, FileResource};

use crate::{ChatHistoryStore, ChatHostState, ChatOutputExecutor};

use super::request;

fn host_with_draft(
    text: &str,
    attachments: Vec<Attachment>,
) -> Result<(ChatHostState, tempfile::TempDir), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut session = ChatSession::new();
    session.draft_mut().set_text(text);
    for attachment in attachments {
        session.draft_mut().add_attachment(attachment);
    }
    let host = ChatHostState::new(
        session,
        ChatHistoryStore::new(root.path().to_path_buf()),
        ChatOutputExecutor::new(root.path().to_path_buf()),
    )
    .map_err(|error| error.to_string())?;
    Ok((host, root))
}

#[test]
fn request_prompt_forwards_text_and_file_attachments() -> Result<(), String> {
    let file = Attachment::file(FileResource::new(
        "file://tmp/report.md",
        "text/markdown",
        "file body",
        9,
    ));
    let (host, root) = host_with_draft(
        "Review these inputs",
        vec![Attachment::text("notes.txt", "text body"), file],
    )?;
    let request = request(&host, root.path())?;
    assert_eq!(
        request.prompt,
        "Review these inputs\n\nAttached text: notes.txt\ntext body\n\nAttached file: file://tmp/report.md\nfile body"
    );
    Ok(())
}

#[test]
fn attachment_only_request_forwards_text_attachment() -> Result<(), String> {
    let (host, root) = host_with_draft("", vec![Attachment::text("selection", "selected text")])?;
    let request = request(&host, root.path())?;
    assert_eq!(request.context.uri, "katanagent://chat");
    assert_eq!(
        request.prompt,
        "\n\nAttached text: selection\nselected text"
    );
    Ok(())
}

#[test]
fn text_attachment_does_not_change_markdown_context_selection() -> Result<(), String> {
    let (host, root) = host_with_draft(
        "tmp/report.md を作成",
        vec![Attachment::text("selection", "selected text")],
    )?;
    let request = request(&host, root.path())?;
    assert_eq!(request.context.uri, "file://tmp/report.md");
    assert!(
        request
            .prompt
            .contains("Attached text: selection\nselected text")
    );
    Ok(())
}

#[test]
fn historical_attachment_only_turn_preserves_text_and_file_contents() -> Result<(), String> {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    session
        .draft_mut()
        .add_attachment(Attachment::text("past notes", "previous text content"));
    session
        .draft_mut()
        .add_attachment(Attachment::file(FileResource::new(
            "file://tmp/previous.md",
            "text/markdown",
            "previous file content",
            21,
        )));
    session.submit_draft().map_err(|error| error.to_string())?;

    let turns = super::history(session.snapshot());
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].role, katana_acp_client::ChatRole::User);
    assert!(turns[0].content.contains("past notes"));
    assert!(turns[0].content.contains("previous text content"));
    assert!(turns[0].content.contains("file://tmp/previous.md"));
    assert!(turns[0].content.contains("text/markdown"));
    assert!(turns[0].content.contains("previous file content"));
    Ok(())
}
