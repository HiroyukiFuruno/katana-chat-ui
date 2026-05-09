use super::{
    ChatOutput, ChatOutputKind, DiffCandidateOutput, FileCandidateOutput, HostActionKind,
    OutputStatus, PermissionRequestOutput,
};

#[test]
fn file_candidate_exposes_create_file_without_writing() {
    let output = ChatOutput::new(
        1,
        10,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "src/generated.rs",
            "text/rust",
            "pub struct Generated;",
        )),
    );

    let actions = output.host_actions();

    assert!(
        actions
            .iter()
            .any(|it| it.kind == HostActionKind::CreateFile)
    );
    assert!(
        actions
            .iter()
            .any(|it| it.kind == HostActionKind::OpenPreview)
    );
}

#[test]
fn diff_candidate_exposes_apply_diff_intent_only() {
    let diff = DiffCandidateOutput::new(
        "src/lib.rs",
        "before",
        "after",
        "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
        "src/lib.rs を更新",
    );
    assert_eq!(diff.original_content, "before");
    assert_eq!(diff.updated_content, "after");
    assert_eq!(diff.summary, "src/lib.rs を更新");

    let output = ChatOutput::new(2, 10, ChatOutputKind::DiffCandidate(diff));

    let actions = output.host_actions();

    assert!(
        actions
            .iter()
            .any(|it| it.kind == HostActionKind::ApplyDiff)
    );
}

#[test]
fn permission_request_exposes_approve_and_reject() {
    let output = ChatOutput::new(
        3,
        10,
        ChatOutputKind::PermissionRequest(PermissionRequestOutput::new(
            "run tests",
            "provider wants to execute tests",
        )),
    );

    let actions = output.host_actions();

    assert!(actions.iter().any(|it| it.kind == HostActionKind::Approve));
    assert!(actions.iter().any(|it| it.kind == HostActionKind::Reject));
}

#[test]
fn applied_file_candidate_exposes_undo_without_create_file() {
    let mut output = ChatOutput::new(
        4,
        10,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/generated.md",
            "text/markdown",
            "# generated",
        )),
    );
    output.status = OutputStatus::Applied;

    let actions = output.host_actions();

    assert!(
        actions
            .iter()
            .any(|it| it.kind == HostActionKind::UndoChange)
    );
    assert!(
        !actions
            .iter()
            .any(|it| it.kind == HostActionKind::CreateFile)
    );
}

#[test]
fn applied_diff_candidate_exposes_undo_without_apply_diff() {
    let mut output = ChatOutput::new(
        5,
        10,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "tmp/sample.md",
            "before",
            "after",
            "--- a/tmp/sample.md\n+++ b/tmp/sample.md\n@@ -1 +1 @@\n-before\n+after",
            "tmp/sample.md を更新",
        )),
    );
    output.status = OutputStatus::Applied;

    let actions = output.host_actions();

    assert!(
        actions
            .iter()
            .any(|it| it.kind == HostActionKind::UndoChange)
    );
    assert!(
        !actions
            .iter()
            .any(|it| it.kind == HostActionKind::ApplyDiff)
    );
}
