use katana_chat_ui::ChatUiHistorySessionSurface;

use crate::ChatHistoryRecord;

pub(crate) fn history_entry(record: &ChatHistoryRecord) -> ChatUiHistorySessionSurface {
    ChatUiHistorySessionSurface {
        session_id: record.session_id.clone(),
        title: history_title(record),
        provider_id: record.provider_id.clone(),
        provider_label: record.provider_id.clone(),
        updated_at_label: format!("updated: {}", record.updated_at_unix_millis),
        preview: history_preview(record),
    }
}

fn history_title(record: &ChatHistoryRecord) -> String {
    let title = record.snapshot.title.trim();
    if title.is_empty() {
        "New chat".to_string()
    } else {
        title.to_string()
    }
}

fn history_preview(record: &ChatHistoryRecord) -> String {
    match record.snapshot.messages.last() {
        Some(message) => message.content.clone(),
        None => "No messages".to_string(),
    }
}
