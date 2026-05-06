use crate::state::ManualHostState;
use eframe::egui;
use katana_chat_ui::{ChatOutputKind, HostActionKind, OutputRenderModel};

pub struct ManualOutputRenderer;

impl ManualOutputRenderer {
    pub fn render(ui: &mut egui::Ui, state: &mut ManualHostState, last_error: &mut Option<String>) {
        let outputs = state.render_model().outputs;
        ui.group(|ui| {
            ui.label("出力物（output）");
            if outputs.is_empty() {
                ui.label("まだ output はありません");
                return;
            }
            for output in outputs {
                Self::output_row(ui, state, last_error, output);
            }
        });
    }

    fn output_row(
        ui: &mut egui::Ui,
        state: &mut ManualHostState,
        last_error: &mut Option<String>,
        output: OutputRenderModel,
    ) {
        ui.separator();
        ui.label(Self::kind_label(&output.kind));
        ui.label(format!("status: {:?}", output.status));
        ui.horizontal_wrapped(|ui| {
            for action in output.actions {
                if ui.button(Self::action_label(action.kind)).clicked() {
                    let result = state.perform_host_action(output.id, action.kind);
                    Self::store_result(result, last_error);
                }
            }
        });
    }

    fn kind_label(kind: &ChatOutputKind) -> String {
        match kind {
            ChatOutputKind::Text(_) => "文章 output".to_string(),
            ChatOutputKind::Code(_) => "code output".to_string(),
            ChatOutputKind::FileCandidate(file) => {
                format!("生成ファイル候補: {}", file.path)
            }
            ChatOutputKind::DiffCandidate(diff) => {
                format!("差分候補: {}", diff.target_path)
            }
            ChatOutputKind::ToolResult(tool) => {
                format!("tool result: {}", tool.tool_name)
            }
            ChatOutputKind::PermissionRequest(permission) => {
                format!("許可待ち: {}", permission.action_label)
            }
        }
    }

    fn action_label(kind: HostActionKind) -> &'static str {
        match kind {
            HostActionKind::Copy => "コピー",
            HostActionKind::OpenPreview => "プレビュー",
            HostActionKind::CreateFile => "ファイル作成",
            HostActionKind::ApplyDiff => "差分適用",
            HostActionKind::Approve => "許可",
            HostActionKind::Reject => "拒否",
        }
    }

    fn store_result(
        result: Result<(), katana_chat_ui::ChatSessionError>,
        last_error: &mut Option<String>,
    ) {
        match result {
            Ok(()) => *last_error = None,
            Err(error) => *last_error = Some(error.to_string()),
        }
    }
}
