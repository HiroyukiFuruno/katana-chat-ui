use super::{ChatUiAction, ChatUiActionError, node};
use crate::ChatUiOutputSurface;
use katana_ui_core::render_model::{UiCommonProps, UiDimension, UiNode, UiNodeKind};

#[cfg(test)]
mod tests;

const OUTPUT_GAP: u16 = 4;
const OUTPUT_ACTION_HEIGHT: u16 = 32;
const OUTPUT_ACTION_WIDTH: u16 = 128;

pub(super) fn output_node(
    surface: &ChatUiOutputSurface,
    node_id: &str,
) -> Result<UiNode, ChatUiActionError> {
    let mut output = output_container(node_id).child(output_body(surface, node_id));
    if let Some(label) = status_label(&surface.status) {
        output = output.child(node::text(
            &label,
            &format!("{node_id}.status"),
            "output-status",
        ));
    }
    for (index, action) in surface.actions.iter().enumerate() {
        output = output.child(action_node(action, node_id, index)?);
    }
    Ok(output)
}

fn output_container(node_id: &str) -> UiNode {
    node::container(UiNodeKind::Column, "", node_id)
        .common(UiCommonProps::default().gap(UiDimension::px(OUTPUT_GAP)))
}

fn output_body(surface: &ChatUiOutputSurface, node_id: &str) -> UiNode {
    node::text(
        &output_label(surface),
        &format!("{node_id}.body"),
        "output-body",
    )
}

fn status_label(status: &crate::OutputStatus) -> Option<String> {
    use crate::OutputStatus;
    match status {
        OutputStatus::Ready => None,
        OutputStatus::PendingHost => Some("適用待ち".into()),
        OutputStatus::Applied => Some("適用済み".into()),
        OutputStatus::Reverted => Some("取り消し済み".into()),
        OutputStatus::Rejected => Some("拒否されました".into()),
        OutputStatus::Failed(reason) => Some(format!("失敗: {reason}")),
    }
}

fn action_node(
    action: &crate::HostActionIntent,
    node_id: &str,
    index: usize,
) -> Result<UiNode, ChatUiActionError> {
    let value = ChatUiAction::Output {
        output_id: action.output_id,
        kind: action.kind,
    };
    node::action(
        UiNodeKind::Button,
        output_action_label(action.kind),
        &format!("{node_id}.action.{index}"),
        &value,
    )
    .map(|button| {
        button
            .height(UiDimension::px(OUTPUT_ACTION_HEIGHT))
            .width(UiDimension::px(OUTPUT_ACTION_WIDTH))
    })
}

fn output_label(surface: &ChatUiOutputSurface) -> String {
    match &surface.kind {
        crate::ChatOutputKind::Text(value) => value.text.clone(),
        crate::ChatOutputKind::Code(value) => code_label(value),
        crate::ChatOutputKind::FileCandidate(value) => {
            format!("{} {}\n{}", value.path, value.mime_type, value.content)
        }
        crate::ChatOutputKind::DiffCandidate(value) => format!(
            "{}\n{}\n{}\n{}\n{}",
            value.summary,
            value.target_path,
            value.unified_diff,
            value.original_content,
            value.updated_content
        ),
        crate::ChatOutputKind::ToolResult(value) => {
            format!("{}: {}", value.tool_name, value.summary)
        }
        crate::ChatOutputKind::PermissionRequest(value) => {
            format!("{}: {}", value.action_label, value.reason)
        }
    }
}

fn code_label(value: &crate::CodeOutput) -> String {
    match &value.language {
        Some(language) => format!("{language}\n{}", value.code),
        None => value.code.clone(),
    }
}

fn output_action_label(kind: crate::HostActionKind) -> &'static str {
    match kind {
        crate::HostActionKind::Copy => "Copy",
        crate::HostActionKind::OpenPreview => "Open preview",
        crate::HostActionKind::CreateFile => "Create file",
        crate::HostActionKind::ApplyDiff => "Apply diff",
        crate::HostActionKind::UndoChange => "Undo change",
        crate::HostActionKind::Approve => "Approve",
        crate::HostActionKind::Reject => "Reject",
    }
}
