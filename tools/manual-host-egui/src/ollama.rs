use katana_acp_client::ollama::OllamaProvider;
use katana_acp_client::{AiIntent, AiProvider, AiRequest, ChatRole, ChatTurn, DocumentContext};

pub struct ManualOllamaClient;

impl ManualOllamaClient {
    pub fn execute(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<String, String> {
        let prompt = prompt.into();
        let provider = Self::provider(endpoint.into(), model.into())?;
        let request = Self::request(prompt);
        let runtime = Self::runtime()?;
        let response = runtime
            .block_on(provider.execute(&request))
            .map_err(|it| it.to_string())?;
        Ok(response.content)
    }

    fn provider(endpoint: String, model: String) -> Result<OllamaProvider, String> {
        if endpoint.trim().is_empty() {
            return Err("Ollama endpoint を入力してください".to_string());
        }
        if model.trim().is_empty() {
            return Err("Ollama model を入力してください".to_string());
        }
        OllamaProvider::new(Some(endpoint), Some(model)).map_err(|it| it.to_string())
    }

    fn request(prompt: String) -> AiRequest {
        AiRequest {
            intent: AiIntent::Create,
            context: DocumentContext {
                uri: "file:///tmp/kcu-manual-ollama.md".to_string(),
                content: "# katana-chat-ui manual host\n".to_string(),
                cursor_offset: 0,
                diagnostics: Vec::new(),
            },
            prompt,
            history: vec![ChatTurn {
                role: ChatRole::System,
                content: "Reply briefly for UI verification.".to_string(),
            }],
        }
    }

    fn runtime() -> Result<tokio::runtime::Runtime, String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|it| it.to_string())
    }
}
