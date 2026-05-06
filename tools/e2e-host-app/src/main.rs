use katana_chat_ui::{
    Attachment, ChatInputDraft, ChatOutputKind, ChatSession, ChatUiSurface, ContextUsageSnapshot,
    DiffCandidateOutput, FileCandidateOutput, FileResource, HostActionKind, OutputRenderModel,
    OutputStatus, VendorUiState,
};
use katana_chat_ui_floem::{FloemChatActions, FloemChatView};

struct HostFixture;

impl HostFixture {
    fn run(&self) -> Result<HostReport, katana_chat_ui::ChatSessionError> {
        let mut session = ChatSession::new();
        session.set_provider_configured("local provider");
        session.set_context_usage(ContextUsageSnapshot::new(750, 1000));
        session.set_vendor_ui_state(Self::vendor_state());
        session.draft_mut().set_text("summarize");
        session
            .draft_mut()
            .add_attachment(Attachment::file(FileResource::new(
                "file:///tmp/kcu-fixture.md",
                "text/markdown",
                "# Fixture",
                9,
            )));

        let ready = session.render_model();
        let path_drop = ChatInputDraft::request_path_drop("/tmp/kcu-fixture.md", "text/markdown");
        session.submit_draft()?;
        session.start_assistant_stream("reading")?;
        let streaming = session.render_model();
        session.finish_assistant_message()?;
        let assistant_id = session.start_assistant_stream("handoff")?;
        session.finish_assistant_message()?;
        let output_id = session.add_output(
            assistant_id,
            ChatOutputKind::FileCandidate(FileCandidateOutput::new(
                "tmp/generated.md",
                "text/markdown",
                "# generated",
            )),
        )?;
        session.set_output_status(output_id, OutputStatus::PendingHost)?;
        session.add_output(
            assistant_id,
            ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                "src/lib.rs",
                "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
            )),
        )?;
        let model = session.render_model();
        let surface = ChatUiSurface::from_render_model(&model);
        let handoff_json = output_json(&model.outputs);
        Self::smoke_construct_floem_widget(surface.clone());

        Ok(HostReport::from_render_model(
            &ready,
            &streaming,
            path_drop.uri,
            &surface,
            &model.outputs,
            &handoff_json,
        ))
    }

    fn vendor_state() -> VendorUiState {
        VendorUiState::for_vendor("ollama")
            .with_endpoint("http://localhost:11434")
            .with_models(vec!["host-model".to_string()], "host-model")
            .with_thinking(vec!["false".to_string(), "medium".to_string()], "medium")
    }

    fn smoke_construct_floem_widget(surface: ChatUiSurface) {
        let surface = floem::prelude::RwSignal::new(surface);
        let draft = floem::prelude::RwSignal::new(String::new());
        let _view = FloemChatView::render(
            surface,
            draft,
            FloemChatActions::new(|| {}, |_| {}, || {}, |_| {}, || {}, |_| {}, |_, _| {}),
        );
    }
}

struct HostReport {
    lines: Vec<String>,
}

impl HostReport {
    fn from_render_model(
        ready_model: &katana_chat_ui::ChatRenderModel,
        model: &katana_chat_ui::ChatRenderModel,
        path_drop_uri: String,
        surface: &ChatUiSurface,
        outputs: &[OutputRenderModel],
        handoff_json: &str,
    ) -> Self {
        let block_count = model.messages.first().map_or(0, |it| it.blocks.len());
        let attachment_count = model.messages.first().map_or(0, |it| it.attachments.len());
        let output_actions = Self::output_action_count(outputs, HostActionKind::CreateFile)
            + Self::output_action_count(outputs, HostActionKind::ApplyDiff);
        Self {
            lines: vec![
                "render:panel".to_string(),
                "standard-ui:surface:composer".to_string(),
                "standard-ui:widget:floem".to_string(),
                format!("input:text:{block_count}"),
                format!("attachment:count:{attachment_count}"),
                format!("path-drop:{path_drop_uri}"),
                format!(
                    "send-before-submit:enabled:{}",
                    ready_model.input.can_submit
                ),
                format!("send:enabled:{}", model.input.can_submit),
                format!("stop:visible:{}", model.input.can_cancel),
                format!("usage:{}", model.context_usage.percentage),
                format!("handoff-json:outputs:{}", outputs.len()),
                format!("handoff-json:actions:{output_actions}"),
                format!("handoff-json:bytes:{}", handoff_json.len()),
                format!(
                    "svg-standard:send:{}",
                    surface.composer.send_icon.svg
                ),
                format!("vendor:active:{}", surface.vendor_ui.active_vendor_id),
                format!(
                    "vendor-control:endpoint:{}",
                    surface
                        .vendor_controls
                        .endpoint
                        .as_ref()
                        .and_then(|it| it.selected_value.clone())
                        .unwrap_or_default()
                ),
                format!(
                    "vendor-affordance:model:{}",
                    surface.vendor_ui.model_selector_visible
                ),
                format!(
                    "vendor-affordance:mode:{}",
                    surface.vendor_ui.mode_selector_visible
                ),
                format!(
                    "vendor-affordance:thinking:{}",
                    surface.vendor_ui.thinking_selector_visible
                ),
                format!(
                    "vendor-affordance:permission:{}",
                    surface.vendor_ui.permission_mode_selector_visible
                ),
                format!(
                    "vendor-affordance:tool-approval:{}",
                    surface.vendor_ui.tool_approval_visible
                ),
                format!(
                    "vendor-affordance:web-search:{}",
                    surface.vendor_ui.web_search_visible
                ),
                format!(
                    "vendor-control:usage:{}",
                    surface.vendor_controls.usage_visible
                ),
                format!(
                    "vendor-control:account-usage:{}",
                    surface.vendor_controls.account_usage_visible
                ),
            ],
        }
    }

    fn output_action_count(outputs: &[OutputRenderModel], expected: HostActionKind) -> usize {
        outputs
            .iter()
            .flat_map(|it| it.actions.iter())
            .filter(|it| it.kind == expected)
            .count()
    }

    fn print(&self) {
        for line in &self.lines {
            println!("{line}");
        }
    }
}

fn output_json(outputs: &[OutputRenderModel]) -> String {
    match serde_json::to_string(outputs) {
        Ok(json) => json,
        Err(error) => format!("output JSON serialization failed: {error}"),
    }
}

fn main() -> Result<(), katana_chat_ui::ChatSessionError> {
    let report = HostFixture.run()?;
    report.print();
    Ok(())
}
