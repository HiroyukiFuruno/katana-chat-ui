use crate::{EguiChatController, EguiChatView};
use eframe::egui;
use katana_chat_ui::{
    ChatOutputKind, ChatSession, ChatUiSurface, ContextUsageSnapshot, DiffCandidateOutput,
    VendorUiState,
};

#[derive(Default)]
struct TestController {
    composer_text: String,
    selected_control: Option<(String, String)>,
    selected_history: Option<String>,
    output_action: Option<(u64, katana_chat_ui::HostActionKind)>,
}

impl EguiChatController for TestController {
    fn composer_text_mut(&mut self) -> &mut String {
        &mut self.composer_text
    }

    fn attach(&mut self) {}

    fn submit(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn select_control(&mut self, key: String, value: String) {
        self.selected_control = Some((key, value));
    }

    fn select_history(&mut self, session_id: String) -> Result<(), String> {
        self.selected_history = Some(session_id);
        Ok(())
    }

    fn output(
        &mut self,
        output_id: u64,
        kind: katana_chat_ui::HostActionKind,
    ) -> Result<(), String> {
        self.output_action = Some((output_id, kind));
        Ok(())
    }
}

#[test]
fn kuc_actions_reach_history_and_output_callbacks() -> Result<(), String> {
    use katana_chat_ui::{ChatUiAction, HostActionKind};
    let mut controller = TestController::default();
    controller.handle_action(ChatUiAction::SelectHistory {
        session_id: "saved-session".into(),
    })?;
    controller.handle_action(ChatUiAction::Output {
        output_id: 42,
        kind: HostActionKind::UndoChange,
    })?;
    assert_eq!(
        controller.selected_history.as_deref(),
        Some("saved-session")
    );
    assert_eq!(
        controller.output_action,
        Some((42, HostActionKind::UndoChange))
    );
    Ok(())
}

#[test]
fn render_accepts_standard_chat_surface() -> Result<(), String> {
    let context = egui::Context::default();
    let surface = sample_surface()?;
    let mut controller = TestController::default();
    let mut view = EguiChatView::default();
    let mut result = Ok(());

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        result = view.render(ui, &surface, &mut controller);
    });
    result?;
    consume_rendered_kuc_texture(&mut output, [800, 600])
}

fn consume_rendered_kuc_texture(
    output: &mut egui::FullOutput,
    expected_size: [usize; 2],
) -> Result<(), String> {
    let uploaded = output
        .textures_delta
        .set
        .values()
        .flatten()
        .any(|delta| delta.image.size() == expected_size);
    output.textures_delta.clear();
    if uploaded {
        Ok(())
    } else {
        Err("render did not upload the KUC texture".into())
    }
}

fn sample_surface() -> Result<ChatUiSurface, String> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("claude-code")
            .with_available_vendors(vec!["claude-code".to_string()])
            .with_models(vec!["claude-sonnet-4-6".to_string()], "claude-sonnet-4-6")
            .with_thinking(vec!["default".to_string()], "default")
            .with_permission_modes(vec!["default".to_string()], "default"),
    );
    session.set_context_usage(ContextUsageSnapshot::new(20, 100));
    session.draft_mut().set_text("hello from egui adapter");
    session.submit_draft().map_err(|it| it.to_string())?;
    let assistant_id = session
        .start_assistant_stream("adapter response")
        .map_err(|it| it.to_string())?;
    session
        .add_output(assistant_id, ChatOutputKind::DiffCandidate(sample_diff()))
        .map_err(|it| it.to_string())?;
    session
        .finish_assistant_message()
        .map_err(|it| it.to_string())?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn sample_diff() -> DiffCandidateOutput {
    DiffCandidateOutput::new(
        "tmp/example.md",
        "before",
        "after",
        "--- a/tmp/example.md\n+++ b/tmp/example.md\n@@ -1 +1 @@\n-before\n+after",
        "tmp/example.md を更新",
    )
}
