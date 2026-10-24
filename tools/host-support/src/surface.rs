use katana_chat_ui::{ChatUiHistorySessionSurface, ChatUiSurface, HostActionKind};

use crate::ChatHistoryRecord;

pub(crate) fn restrict_output_actions(surface: &mut ChatUiSurface) {
    // 外部host向けのintentは維持し、標準hostが処理できる操作だけを提示する。
    for output in surface
        .messages
        .iter_mut()
        .chain(surface.message_list.messages.iter_mut())
        .flat_map(|message| message.outputs.iter_mut())
        .chain(surface.output_handoff.outputs.iter_mut())
    {
        output.actions.retain(|action| {
            matches!(
                action.kind,
                HostActionKind::CreateFile | HostActionKind::ApplyDiff | HostActionKind::UndoChange
            )
        });
    }
}

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
