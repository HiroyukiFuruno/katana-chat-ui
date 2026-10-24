use super::{
    Attachment, AttachmentPolicy, ChatInputDraft, FileResource, ImageResource, PathDropRequest,
};
use katana_acp_client::AcpContentBlock;

#[test]
fn draft_converts_text_and_attachments_to_acp_content_blocks() {
    let mut draft = ChatInputDraft::new();
    draft.set_text("Summarize this file");
    draft.add_attachment(Attachment::file(FileResource::new(
        "file:///tmp/note.md",
        "text/markdown",
        "# Note",
        6,
    )));
    draft.add_attachment(Attachment::image(ImageResource::new(
        "image/png",
        "host-image://1",
        42,
    )));

    let blocks = draft.to_acp_content_blocks();

    assert!(matches!(blocks[0], AcpContentBlock::Text(_)));
    assert!(matches!(blocks[1], AcpContentBlock::EmbeddedResource(_)));
    assert!(matches!(blocks[2], AcpContentBlock::Image(_)));
}

#[test]
fn path_drop_returns_request_without_reading_filesystem() {
    let request = ChatInputDraft::request_path_drop("/tmp/notes.md", "text/markdown");

    assert_eq!(
        request,
        PathDropRequest::new("file:///tmp/notes.md", "text/markdown")
    );
}

#[test]
fn image_policy_reports_unsupported_attachment_before_submit() {
    let mut draft = ChatInputDraft::new();
    draft.add_attachment(Attachment::image(ImageResource::new(
        "image/png",
        "host-image://1",
        42,
    )));

    let result = draft.validate_attachments(&AttachmentPolicy::text_and_files_only(
        "provider does not support images",
    ));

    assert_eq!(result.unsupported_reasons().len(), 1);
}

#[test]
fn draft_removes_attachment_by_index() {
    let mut draft = ChatInputDraft::new();
    draft.add_attachment(Attachment::text("first.md", "first"));
    draft.add_attachment(Attachment::text("second.md", "second"));

    let removed = draft.remove_attachment(0);

    assert!(removed.is_some());
    assert_eq!(draft.attachments.len(), 1);
    assert_eq!(
        draft.attachments,
        vec![Attachment::text("second.md", "second")]
    );
}
