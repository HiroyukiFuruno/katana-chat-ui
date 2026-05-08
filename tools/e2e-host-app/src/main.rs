use katana_chat_ui::{
    Attachment, ChatInputDraft, ChatOutputKind, ChatRenderModel, ChatSession, ChatUiSurface,
    ContextUsageSnapshot, DiffCandidateOutput, FileCandidateOutput, FileResource, HostActionKind,
    OutputRenderModel, OutputStatus, VendorUiState,
};
use katana_chat_ui_floem::{FloemChatActions, FloemChatView};

struct HostFixture;

impl HostFixture {
    fn run(&self) -> Result<HostReport, katana_chat_ui::ChatSessionError> {
        let mut session = Self::session();
        let ready = session.render_model();
        let path_drop_uri = Self::path_drop_uri();
        let streaming = Self::submit_and_capture_streaming(&mut session)?;
        let assistant_id = Self::finish_and_start_handoff(&mut session)?;
        Self::add_outputs(&mut session, assistant_id)?;

        let model = session.render_model();
        let surface = ChatUiSurface::from_render_model(&model);
        let handoff_json = output_json(&model.outputs);
        Self::smoke_construct_floem_widget(surface.clone());

        Ok(HostReport::from_render_model(
            &ready,
            &streaming,
            path_drop_uri,
            &surface,
            &model.outputs,
            &handoff_json,
        ))
    }

    fn session() -> ChatSession {
        let mut session = ChatSession::new();
        session.set_provider_configured("Claude Code");
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
        session
    }

    fn path_drop_uri() -> String {
        let path_drop = ChatInputDraft::request_path_drop("/tmp/kcu-fixture.md", "text/markdown");
        path_drop.uri
    }

    fn submit_and_capture_streaming(
        session: &mut ChatSession,
    ) -> Result<ChatRenderModel, katana_chat_ui::ChatSessionError> {
        session.submit_draft()?;
        session.start_assistant_stream("reading")?;
        let streaming = session.render_model();
        session.finish_assistant_message()?;
        Ok(streaming)
    }

    fn finish_and_start_handoff(
        session: &mut ChatSession,
    ) -> Result<u64, katana_chat_ui::ChatSessionError> {
        let assistant_id = session.start_assistant_stream("handoff")?;
        session.finish_assistant_message()?;
        Ok(assistant_id)
    }

    fn add_outputs(
        session: &mut ChatSession,
        assistant_id: u64,
    ) -> Result<(), katana_chat_ui::ChatSessionError> {
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
                "before",
                "after",
                "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
                "src/lib.rs を更新",
            )),
        )?;
        Ok(())
    }

    fn vendor_state() -> VendorUiState {
        VendorUiState::for_vendor("claude-code")
            .with_available_vendors(vec![
                "claude-code".to_string(),
                "codex-cli".to_string(),
                "github-copilot".to_string(),
                "opencode".to_string(),
            ])
            .with_models(vec!["claude-sonnet-4-6".to_string()], "claude-sonnet-4-6")
            .with_thinking(vec!["default".to_string(), "medium".to_string()], "default")
            .with_permission_modes(vec!["default".to_string(), "auto".to_string()], "default")
    }

    fn smoke_construct_floem_widget(surface: ChatUiSurface) {
        let surface = floem::prelude::RwSignal::new(surface);
        let draft = floem::prelude::RwSignal::new(String::new());
        let _view = FloemChatView::render(
            surface,
            draft,
            FloemChatActions {
                on_attach: || {},
                on_remove_attachment: |_| {},
                on_new_chat: || {},
                on_history: || {},
                on_submit: |_| {},
                on_stop: || {},
                on_vendor_select: |_| {},
                on_control_select: |_, _| {},
            },
        );
    }
}

struct HostReport {
    lines: Vec<String>,
}

impl HostReport {
    fn from_render_model(
        ready_model: &ChatRenderModel,
        model: &ChatRenderModel,
        path_drop_uri: String,
        surface: &ChatUiSurface,
        outputs: &[OutputRenderModel],
        handoff_json: &str,
    ) -> Self {
        let mut lines = Vec::new();
        Self::push_standard_lines(&mut lines, ready_model, model, path_drop_uri);
        Self::push_handoff_lines(&mut lines, outputs, handoff_json);
        Self::push_vendor_lines(&mut lines, surface);
        Self { lines }
    }

    fn push_standard_lines(
        lines: &mut Vec<String>,
        ready_model: &ChatRenderModel,
        model: &ChatRenderModel,
        path_drop_uri: String,
    ) {
        let block_count = model.messages.first().map_or(0, |it| it.blocks.len());
        let attachment_count = model.messages.first().map_or(0, |it| it.attachments.len());
        lines.extend([
            "render:panel".to_string(),
            "standard-ui:surface:composer".to_string(),
            "standard-ui:widget:floem".to_string(),
            format!("input:text:{block_count}"),
            format!("attachment:count:{attachment_count}"),
            format!("attachment:path_drop:{path_drop_uri}"),
            format!("path-drop:{path_drop_uri}"),
            format!(
                "send-before-submit:enabled:{}",
                ready_model.input.can_submit
            ),
            format!("send:enabled:{}", model.input.can_submit),
            format!("stop:visible:{}", model.input.can_cancel),
            format!("usage:{}", model.context_usage.percentage),
        ]);
    }

    fn push_handoff_lines(
        lines: &mut Vec<String>,
        outputs: &[OutputRenderModel],
        handoff_json: &str,
    ) {
        let output_actions = Self::output_action_count(outputs, HostActionKind::CreateFile)
            + Self::output_action_count(outputs, HostActionKind::ApplyDiff);
        lines.extend([
            format!("handoff-json:outputs:{}", outputs.len()),
            format!("handoff-json:actions:{output_actions}"),
            format!("handoff-json:bytes:{}", handoff_json.len()),
        ]);
    }

    fn push_vendor_lines(lines: &mut Vec<String>, surface: &ChatUiSurface) {
        Self::push_vendor_identity_lines(lines, surface);
        Self::push_vendor_affordance_lines(lines, surface);
        Self::push_vendor_usage_lines(lines, surface);
    }

    fn push_vendor_identity_lines(lines: &mut Vec<String>, surface: &ChatUiSurface) {
        lines.extend([
            format!("svg-standard:send:{}", surface.composer.send_icon.svg),
            format!("vendor:active:{}", surface.vendor_ui.active_vendor_id),
            format!("vendor-control:endpoint:{}", endpoint_value(surface)),
        ]);
    }

    fn push_vendor_affordance_lines(lines: &mut Vec<String>, surface: &ChatUiSurface) {
        lines.extend([
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
        ]);
    }

    fn push_vendor_usage_lines(lines: &mut Vec<String>, surface: &ChatUiSurface) {
        lines.extend([
            format!(
                "vendor-control:usage:{}",
                surface.vendor_controls.usage_visible
            ),
            format!(
                "vendor-control:account-usage:{}",
                surface.vendor_controls.account_usage_visible
            ),
        ]);
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

fn endpoint_value(surface: &ChatUiSurface) -> String {
    surface
        .vendor_controls
        .endpoint
        .as_ref()
        .and_then(|it| it.selected_value.clone())
        .unwrap_or_default()
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
