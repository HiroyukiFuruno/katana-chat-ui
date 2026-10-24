use std::path::Path;

use katana_acp_client::{AiIntent, AiRequest, ChatRole, ChatTurn};
use katana_chat_ui::{Attachment, ChatSessionSnapshot, MessageRole};

use crate::ChatHostState;

mod context;

#[cfg(test)]
mod tests;

pub(super) fn request(host: &ChatHostState, workspace_root: &Path) -> Result<AiRequest, String> {
    let snapshot = host.session().snapshot();
    let command = snapshot.draft.text.clone();
    let context = context::select(&snapshot.draft.attachments, &command, workspace_root)?;
    let prompt = prompt(&snapshot, &context.document.uri);
    Ok(AiRequest {
        intent: context.intent,
        context: context.document,
        prompt,
        history: history(snapshot),
    })
}

pub(super) fn is_markdown_request(request: &AiRequest) -> bool {
    request.context.uri.starts_with("file://tmp/")
        && matches!(request.intent, AiIntent::Create | AiIntent::Modify)
}

fn prompt(snapshot: &ChatSessionSnapshot, context_uri: &str) -> String {
    let mut prompt = snapshot.draft.text.clone();
    for attachment in &snapshot.draft.attachments {
        append_attachment_context(&mut prompt, attachment);
    }
    if context_uri.starts_with("file://tmp/") {
        prompt.push_str("\n\n対象の変更後 Markdown 本文のみを返してください。説明文やコードフェンスは含めないでください。");
    }
    prompt
}

fn append_attachment_context(prompt: &mut String, attachment: &Attachment) {
    match attachment {
        Attachment::Text(text) => {
            prompt.push_str("\n\nAttached text: ");
            prompt.push_str(&text.label);
            prompt.push('\n');
            prompt.push_str(&text.text);
        }
        Attachment::FileResource(file) => {
            prompt.push_str("\n\nAttached file: ");
            prompt.push_str(&file.uri);
            prompt.push('\n');
            prompt.push_str(&file.text);
        }
        Attachment::ImageResource(_) | Attachment::Unsupported(_) => {}
    }
}

fn history(snapshot: ChatSessionSnapshot) -> Vec<ChatTurn> {
    snapshot
        .messages
        .into_iter()
        .map(|message| {
            let mut content = message.content;
            for attachment in &message.attachments {
                append_attachment_context(&mut content, attachment);
                if let Attachment::FileResource(file) = attachment {
                    content.push_str("\nMIME type: ");
                    content.push_str(&file.mime_type);
                }
            }
            crate::history_output::append_for_message(&mut content, message.id, &snapshot.outputs);
            ChatTurn {
                role: role(message.role),
                content,
            }
        })
        .collect()
}

fn role(role: MessageRole) -> ChatRole {
    match role {
        MessageRole::User => ChatRole::User,
        MessageRole::Assistant => ChatRole::Assistant,
        MessageRole::Tool => ChatRole::Tool,
        MessageRole::System => ChatRole::System,
    }
}
