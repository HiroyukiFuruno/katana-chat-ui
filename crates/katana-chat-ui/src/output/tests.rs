use super::{
    ChatOutput, ChatOutputKind, DiffCandidateOutput, FileCandidateOutput, HostActionKind,
    PermissionRequestOutput,
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
    let output = ChatOutput::new(
        2,
        10,
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "src/lib.rs",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
        )),
    );

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
