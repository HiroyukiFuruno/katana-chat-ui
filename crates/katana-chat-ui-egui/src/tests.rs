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
}

#[test]
fn render_accepts_standard_chat_surface() -> Result<(), String> {
    let context = egui::Context::default();
    let surface = sample_surface()?;
    let mut controller = TestController::default();

    let _output = context.run_ui(egui::RawInput::default(), |ui| {
        EguiChatView::render(ui, &surface, &mut controller);
    });

    Ok(())
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
        .add_output(
            assistant_id,
            ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                "tmp/example.md",
                "--- a/tmp/example.md\n+++ b/tmp/example.md\n@@ -1 +1 @@\n-before\n+after",
            )),
        )
        .map_err(|it| it.to_string())?;
    session
        .finish_assistant_message()
        .map_err(|it| it.to_string())?;
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}
