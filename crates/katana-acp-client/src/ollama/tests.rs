use super::{
    OllamaProvider,
    request::OllamaChatRequestBuilder,
    stream::OllamaStreamDecoder,
    types::{OllamaChatRequest, OllamaThink},
};
use crate::{AcpError, AiIntent, AiRequest, AiStreamEvent, ChatRole, ChatTurn, DocumentContext};

#[test]
fn test_endpoint_normalization() -> Result<(), AcpError> {
    let default_provider = OllamaProvider::new(None, None)?;
    assert_eq!(default_provider.endpoint, "http://localhost:11434");

    let custom_provider = OllamaProvider::new(Some("http://127.0.0.1:11434/".to_string()), None)?;
    assert_eq!(custom_provider.endpoint, "http://127.0.0.1:11434");
    Ok(())
}

#[test]
fn stream_parser_emits_each_message_content_chunk() -> Result<(), AcpError> {
    let mut pending = String::new();
    let mut chunks = Vec::new();

    OllamaStreamDecoder::append(
        &mut pending,
        br#"{"message":{"role":"assistant","content":"Hel"}}"#,
        &mut |chunk| {
            chunks.push(chunk);
            Ok(())
        },
    )?;
    OllamaStreamDecoder::append(
        &mut pending,
        b"\n{\"message\":{\"role\":\"assistant\",\"content\":\"lo\"}}\n{\"done\":true}\n",
        &mut |chunk| {
            chunks.push(chunk);
            Ok(())
        },
    )?;

    assert_eq!(chunks, vec!["Hel".to_string(), "lo".to_string()]);
    assert!(pending.is_empty());
    Ok(())
}

#[test]
fn stream_parser_emits_thinking_and_content_separately() -> Result<(), AcpError> {
    let mut pending = String::new();
    let mut events = Vec::new();

    append_event(&mut pending, &mut events, "", "検討")?;
    append_event(&mut pending, &mut events, "結果", "")?;

    assert_eq!(
        events,
        vec![
            AiStreamEvent::Thinking("検討".to_string()),
            AiStreamEvent::Content("結果".to_string())
        ]
    );
    Ok(())
}

#[test]
fn chat_request_builds_model_stream_and_thinking_effort() {
    let request = request_with_history();

    let chat = request_with_low_effort(&request);

    assert_eq!(chat.model, "gemma4:e4b");
    assert!(chat.stream);
    assert!(matches!(chat.think, Some(OllamaThink::Effort(value)) if value == "low"));
}

#[test]
fn chat_request_builds_system_context() {
    let request = request_with_history();

    let chat = request_with_low_effort(&request);

    assert_eq!(chat.messages.len(), 4);
    assert_eq!(chat.messages[0].role, "system");
    assert!(chat.messages[0].content.contains("System guardrail"));
    assert!(chat.messages[0].content.contains("selected document"));
}

#[test]
fn chat_request_builds_history_and_user_prompt() {
    let request = request_with_history();

    let chat = request_with_low_effort(&request);

    assert_eq!(chat.messages[1].role, "user");
    assert_eq!(chat.messages[2].role, "assistant");
    assert_eq!(chat.messages[3].content, "Summarize this");
}

#[test]
fn chat_request_maps_thinking_false_to_boolean() {
    let request = request_with_history();

    let chat = OllamaChatRequestBuilder::build(
        "llama3",
        &request,
        "context".to_string(),
        false,
        Some("false".to_string()),
    );

    assert!(!chat.stream);
    assert!(matches!(chat.think, Some(OllamaThink::Enabled(false))));
}

#[test]
fn chat_request_filters_system_history_from_regular_messages() {
    let request = request_with_history();

    let chat =
        OllamaChatRequestBuilder::build("llama3", &request, "context".to_string(), true, None);

    assert_eq!(
        chat.messages
            .iter()
            .filter(|message| message.role == "system")
            .count(),
        1
    );
}

fn append_event(
    pending: &mut String,
    events: &mut Vec<AiStreamEvent>,
    content: &str,
    thinking: &str,
) -> Result<(), AcpError> {
    let event_json = format!(
        "{{\"message\":{{\"role\":\"assistant\",\"content\":\"{content}\",\"thinking\":\"{thinking}\"}}}}\n"
    );
    OllamaStreamDecoder::append_event(pending, event_json.as_bytes(), &mut |event| {
        events.push(event);
        Ok(())
    })
}

fn request_with_history() -> AiRequest {
    AiRequest {
        intent: AiIntent::Create,
        context: DocumentContext {
            uri: "file:///tmp/sample.md".to_string(),
            content: "# Sample".to_string(),
            cursor_offset: 3,
            diagnostics: vec!["missing title".to_string()],
        },
        prompt: "Summarize this".to_string(),
        history: vec![
            ChatTurn {
                role: ChatRole::System,
                content: "System guardrail".to_string(),
            },
            ChatTurn {
                role: ChatRole::User,
                content: "hello".to_string(),
            },
            ChatTurn {
                role: ChatRole::Assistant,
                content: "hi".to_string(),
            },
        ],
    }
}

fn request_with_low_effort(request: &AiRequest) -> OllamaChatRequest {
    OllamaChatRequestBuilder::build(
        "gemma4:e4b",
        request,
        "selected document".to_string(),
        true,
        Some("low".to_string()),
    )
}
