use crate::{ChatOutputKind, ChatSession, ChatUiSurface, FileCandidateOutput, OutputStatus};

mod raster;

#[test]
fn output_result_reaches_the_visible_kuc_status_node() -> Result<(), Box<dyn std::error::Error>> {
    for (status, expected) in [
        (OutputStatus::Applied, "適用済み"),
        (OutputStatus::Reverted, "取り消し済み"),
        (
            OutputStatus::Failed("content changed".into()),
            "失敗: content changed",
        ),
    ] {
        let surface = surface(status)?;
        let output = surface
            .output_handoff
            .outputs
            .first()
            .ok_or("output missing")?;
        let node = super::output_node(output, "test.output")?;
        let status = node
            .children()
            .iter()
            .find(|child| child.id().as_str() == "test.output.status")
            .ok_or("visible status missing")?;
        assert_eq!(status.props().label, expected);
    }
    Ok(())
}

fn surface(status: OutputStatus) -> Result<ChatUiSurface, crate::ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("KatanAgent");
    let message = session.start_assistant_stream("result")?;
    let output = session.add_output(
        message,
        ChatOutputKind::FileCandidate(FileCandidateOutput::new(
            "tmp/a.md",
            "text/markdown",
            "# result",
        )),
    )?;
    session.set_output_status(output, status)?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}
