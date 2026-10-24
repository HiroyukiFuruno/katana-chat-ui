use crate::{AiRequest, ChatRole};

pub(super) struct OllamaPromptBuilder;

impl OllamaPromptBuilder {
    pub(super) fn system_message(request: &AiRequest, context_message: String) -> String {
        let mut system_message = request
            .history
            .iter()
            .find(|turn| turn.role == ChatRole::System)
            .map(|turn| turn.content.clone())
            .unwrap_or_else(String::new);
        if !system_message.is_empty() {
            system_message.push_str("\n\n");
        }
        system_message.push_str("Current Document Context:\n");
        system_message.push_str(&context_message);
        system_message.push_str("\n\nIntent: ");
        system_message.push_str(Self::intent_message(request));
        system_message
    }

    fn intent_message(request: &AiRequest) -> &'static str {
        match request.intent {
            crate::AiIntent::Modify => "Modify existing code.",
            crate::AiIntent::Create => "Create new code or documentation.",
            crate::AiIntent::Autofix => "Automatically fix errors or diagnostics.",
        }
    }
}
