use super::thread_layout::{MessageBubbleLayout, ThreadMessagePresenter};
use katana_chat_ui::{
    AgentActivityKind, Attachment, ChatUiActivitySurface, ChatUiMessageAlignment,
    ChatUiMessageSurface, MarkdownSubset, MessageRole, MessageStatus,
};

const THREAD_SOURCE: &str = include_str!("thread.rs");
const STYLE_SOURCE: &str = include_str!("styles.rs");

#[test]
fn thread_source_centers_shared_chat_body_column() {
    assert!(THREAD_SOURCE.contains(".width_full()"));
    assert!(THREAD_SOURCE.contains("max_width(styles::CHAT_BODY_MAX_WIDTH)"));
    assert!(THREAD_SOURCE.contains(".items_center()"));
    assert!(THREAD_SOURCE.contains(".justify_center()"));
}

#[test]
fn root_panel_does_not_shrink_chat_to_child_content_width() {
    assert!(STYLE_SOURCE.contains(".size_full()"));
    assert!(!STYLE_SOURCE.contains(".items_center()"));
}

#[test]
fn visible_body_hides_role_labels() {
    let message = message("User", MessageStatus::Complete, "こんにちは");

    assert_eq!(ThreadMessagePresenter::visible_body(&message), "こんにちは");
}

#[test]
fn visible_body_hides_empty_streaming_status_without_thinking_log() {
    let message = message("Assistant", MessageStatus::Streaming, "");

    assert_eq!(ThreadMessagePresenter::visible_body(&message), "Processing");
}

#[test]
fn bubble_body_preserves_long_assistant_response_without_forced_columns() {
    let body = "これはとても長い応答本文で、固定文字数の強制改行ではなく、吹き出しの表示幅で折り返します。";
    let message = message("Assistant", MessageStatus::Complete, body);

    assert_eq!(ThreadMessagePresenter::bubble_body(&message), body);
}

#[test]
fn bubble_width_uses_stable_percentage_not_content_width() {
    assert_eq!(ThreadMessagePresenter::agent_bubble_width_percent(), 100.0);
}

#[test]
fn thread_and_composer_share_chat_body_width_contract() {
    assert_eq!(ThreadMessagePresenter::chat_body_max_width(), 1600.0);
}

#[test]
fn assistant_answer_uses_agent_fixed_width_layout() {
    let message = message("Assistant", MessageStatus::Complete, "応答しました");

    assert_eq!(
        ThreadMessagePresenter::bubble_layout(&message),
        MessageBubbleLayout::AgentFixed
    );
}

#[test]
fn user_message_uses_content_sized_right_aligned_layout() {
    let mut message = message("User", MessageStatus::Complete, "こんにちは");
    message.role = MessageRole::User;
    message.alignment = ChatUiMessageAlignment::Trailing;

    assert_eq!(
        ThreadMessagePresenter::bubble_layout(&message),
        MessageBubbleLayout::ContentSized
    );
}

#[test]
fn thinking_only_message_uses_state_content_sized_layout() {
    let mut message = message("Assistant", MessageStatus::Streaming, "");
    message.activity = None;
    message.thinking = Some(katana_chat_ui::ChatUiThinkingSurface {
        label: "Thinking".to_string(),
        entries: vec!["read context".to_string()],
        expanded: true,
        completed: false,
    });

    assert_eq!(
        ThreadMessagePresenter::bubble_layout(&message),
        MessageBubbleLayout::StateContentSized
    );
}

#[test]
fn thinking_layout_is_not_treated_as_agent_response_width() {
    let mut message = message("Assistant", MessageStatus::Streaming, "");
    message.activity = None;
    message.thinking = Some(katana_chat_ui::ChatUiThinkingSurface {
        label: "Thinking".to_string(),
        entries: Vec::new(),
        expanded: true,
        completed: false,
    });

    assert_ne!(
        ThreadMessagePresenter::bubble_layout(&message),
        MessageBubbleLayout::AgentFixed
    );
}

#[test]
fn activity_only_message_uses_state_content_sized_layout() {
    let message = message("Assistant", MessageStatus::Streaming, "");

    assert_eq!(
        ThreadMessagePresenter::bubble_layout(&message),
        MessageBubbleLayout::StateContentSized
    );
}

#[test]
fn bubble_vertical_padding_does_not_add_extra_lower_space() {
    assert_eq!(
        ThreadMessagePresenter::bubble_vertical_metrics().padding_y,
        8.0
    );
}

#[test]
fn root_wraps_overlay_stack_with_full_size_container() {
    let source = include_str!("root.rs");

    assert!(source.contains("container(panel"));
    assert!(source.contains(".style(|style| style.size_full())"));
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
fn message_key_changes_when_activity_kind_changes() {
    let before = message("Assistant", MessageStatus::Streaming, "");
    let mut after = message("Assistant", MessageStatus::Streaming, "");
    after.activity = Some(ChatUiActivitySurface {
        kind: AgentActivityKind::Generating,
        label: "Generating".to_string(),
    });

    assert_ne!(
        ThreadMessagePresenter::message_key(&before),
        ThreadMessagePresenter::message_key(&after)
    );
}

#[test]
fn waiting_indicator_requires_empty_streaming_body() {
    let mut message = message("Assistant", MessageStatus::Streaming, "");
    message.activity = None;
    message.thinking = Some(katana_chat_ui::ChatUiThinkingSurface {
        label: "Thinking".to_string(),
        entries: Vec::new(),
        expanded: true,
        completed: false,
    });

    assert!(ThreadMessagePresenter::is_waiting_indicator(&message));
}

#[test]
fn waiting_indicator_accepts_activity_labels_beyond_processing() {
    let mut message = message("Assistant", MessageStatus::Streaming, "");
    message.activity = Some(ChatUiActivitySurface {
        kind: AgentActivityKind::Generating,
        label: "Generating".to_string(),
    });

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
        status_label: "Processing".to_string(),
        activity: Some(ChatUiActivitySurface {
            kind: AgentActivityKind::Processing,
            label: "Processing".to_string(),
        }),
        alignment: ChatUiMessageAlignment::Leading,
        body: body.to_string(),
        blocks: MarkdownSubset::parse(body).blocks,
        thinking: None,
        outputs: Vec::new(),
        attachments: Vec::<Attachment>::new(),
    }
}
