use super::thread::ThreadMessagePresenter;
use katana_chat_ui::{
    Attachment, ChatUiMessageAlignment, ChatUiMessageSurface, MarkdownSubset, MessageRole,
    MessageStatus,
};

#[test]
fn visible_body_hides_role_labels() {
    let message = message("User", MessageStatus::Complete, "こんにちは");

    assert_eq!(ThreadMessagePresenter::visible_body(&message), "こんにちは");
}

#[test]
fn visible_body_exposes_streaming_status_when_body_is_empty() {
    let message = message("Assistant", MessageStatus::Streaming, "");

    assert_eq!(ThreadMessagePresenter::visible_body(&message), "Thinking");
}

#[test]
fn bubble_body_preserves_long_assistant_response_without_forced_columns() {
    let body = "これはとても長い応答本文で、固定文字数の強制改行ではなく、吹き出しの表示幅で折り返します。";
    let message = message("Assistant", MessageStatus::Complete, body);

    assert_eq!(ThreadMessagePresenter::bubble_body(&message), body);
}

#[test]
fn bubble_width_uses_stable_percentage_not_content_width() {
    assert_eq!(ThreadMessagePresenter::bubble_width_percent(), 78.0);
}

#[test]
fn message_key_changes_when_streaming_message_becomes_complete() {
    let before = message("Assistant", MessageStatus::Streaming, "");
    let after = message("Assistant", MessageStatus::Complete, "応答しました");

    assert_ne!(
        ThreadMessagePresenter::message_key(&before),
        ThreadMessagePresenter::message_key(&after)
    );
}

#[test]
fn waiting_indicator_requires_empty_streaming_body() {
    let message = message("Assistant", MessageStatus::Streaming, "");

    assert!(ThreadMessagePresenter::is_waiting_indicator(&message));
}

#[test]
fn waiting_indicator_does_not_hide_streaming_content() {
    let message = message("Assistant", MessageStatus::Streaming, "Hello");

    assert!(!ThreadMessagePresenter::is_waiting_indicator(&message));
}

#[test]
fn waiting_indicator_does_not_replace_error_or_completed_body() {
    let error = message("Assistant", MessageStatus::Error("failed".to_string()), "");
    let complete = message("Assistant", MessageStatus::Complete, "");

    assert!(!ThreadMessagePresenter::is_waiting_indicator(&error));
    assert!(!ThreadMessagePresenter::is_waiting_indicator(&complete));
}

fn message(role_label: &str, status: MessageStatus, body: &str) -> ChatUiMessageSurface {
    ChatUiMessageSurface {
        id: 1,
        role: MessageRole::Assistant,
        role_label: role_label.to_string(),
        status,
        status_label: "Thinking".to_string(),
        alignment: ChatUiMessageAlignment::Leading,
        body: body.to_string(),
        blocks: MarkdownSubset::parse(body).blocks,
        thinking: None,
        outputs: Vec::new(),
        attachments: Vec::<Attachment>::new(),
    }
}
