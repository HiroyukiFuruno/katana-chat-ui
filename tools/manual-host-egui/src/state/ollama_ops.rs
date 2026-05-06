use super::ManualHostState;
use crate::ollama::ManualOllamaClient;
use katana_chat_ui::{ChatOutputKind, ChatSessionError, TextOutput};

impl ManualHostState {
    pub fn run_ollama(&mut self) -> Result<(), String> {
        let prompt = self.composer_text.clone();
        self.submit_user_message().map_err(|it| it.to_string())?;
        let response = ManualOllamaClient::execute(
            self.ollama_endpoint.clone(),
            self.ollama_model.clone(),
            prompt,
        )?;
        self.add_ollama_response(response)
            .map_err(|it| it.to_string())?;
        self.composer_text.clear();
        self.last_action = "Ollama の local LLM 応答を追加しました".to_string();
        Ok(())
    }

    fn add_ollama_response(&mut self, response: String) -> Result<(), ChatSessionError> {
        let assistant_id = self.session.start_assistant_stream(response.clone())?;
        self.session.finish_assistant_message()?;
        self.session.add_output(
            assistant_id,
            ChatOutputKind::Text(TextOutput::new(response)),
        )?;
        Ok(())
    }
}
