use super::prompt::OllamaPromptBuilder;
use super::types::{OllamaChatRequest, OllamaMessage, OllamaThink};
use crate::{AiRequest, ChatRole, ChatTurn};

pub(crate) struct OllamaChatRequestBuilder;

impl OllamaChatRequestBuilder {
    pub(crate) fn build(
        model: &str,
        request: &AiRequest,
        context: String,
        stream: bool,
        thinking: Option<String>,
    ) -> OllamaChatRequest {
        OllamaChatRequest {
            model: model.to_string(),
            messages: Self::messages(request, context),
            stream,
            think: thinking.map(thinking_option),
        }
    }

    fn messages(request: &AiRequest, context: String) -> Vec<OllamaMessage> {
        let mut messages = vec![OllamaMessage {
            role: "system".to_string(),
            content: OllamaPromptBuilder::system_message(request, context),
            thinking: None,
        }];
        messages.extend(request.history.iter().filter_map(Self::history_message));
        messages.push(OllamaMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
            thinking: None,
        });
        messages
    }

    fn history_message(turn: &ChatTurn) -> Option<OllamaMessage> {
        let role = match turn.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::Tool => "tool",
            ChatRole::System => return None,
        };
        Some(OllamaMessage {
            role: role.to_string(),
            content: turn.content.clone(),
            thinking: None,
        })
    }
}

fn thinking_option(value: String) -> OllamaThink {
    match value.as_str() {
        "false" => OllamaThink::Enabled(false),
        "true" => OllamaThink::Enabled(true),
        "low" | "medium" | "high" => OllamaThink::Effort(value),
        _ => OllamaThink::Effort(value),
    }
}
