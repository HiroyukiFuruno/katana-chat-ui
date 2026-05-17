use super::styles;
use floem::{AnyView, peniko::Color, prelude::*};
use katana_chat_ui::{ChatOutputKind, ChatUiOutputSurface, HostActionIntent, HostActionKind};

const CARD_GAP: f64 = 8.0;
const CARD_PADDING: f64 = 12.0;
const PREVIEW_PADDING: f64 = 10.0;
const PREVIEW_MAX_HEIGHT: f64 = 160.0;
const PREVIEW_LIMIT: usize = 640;
const CARD_BACKGROUND: Color = Color::from_rgb8(248, 249, 251);
const PREVIEW_BACKGROUND: Color = Color::from_rgb8(242, 244, 247);

pub(super) struct FloemOutputCardsView;

impl FloemOutputCardsView {
    pub(super) fn render<OnOutputAction>(
        outputs: Vec<ChatUiOutputSurface>,
        on_output_action: OnOutputAction,
    ) -> AnyView
    where
        OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
    {
        if outputs.is_empty() {
            return empty().into_any();
        }
        v_stack_from_iter(
            outputs
                .into_iter()
                .map(move |output| output_card(output, on_output_action)),
        )
        .style(|style| style.width_full().min_width(0.0).gap(CARD_GAP))
        .into_any()
    }
}

fn output_card<OnOutputAction>(
    output: ChatUiOutputSurface,
    on_output_action: OnOutputAction,
) -> AnyView
where
    OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
{
    let actions = output.actions.clone();
    let output_id = output.id;
    let summary = OutputCardSummary::from_output(output);
    v_stack((
        card_header(summary.title),
        card_body(summary.detail),
        action_row(output_id, actions, on_output_action),
    ))
    .style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .padding(CARD_PADDING)
            .gap(CARD_GAP)
            .border(1.0)
            .border_color(styles::COLOR_BORDER)
            .border_radius(styles::BUBBLE_RADIUS)
            .background(CARD_BACKGROUND)
    })
    .into_any()
}

fn card_header(title: String) -> impl IntoView {
    label(move || title.clone())
        .style(|style| style.font_size(styles::FONT_BODY).color(styles::COLOR_TEXT))
}

fn card_body(detail: String) -> impl IntoView {
    scroll(label(move || detail.clone())).style(|style| {
        style
            .width_full()
            .min_width(0.0)
            .max_height(PREVIEW_MAX_HEIGHT)
            .padding(PREVIEW_PADDING)
            .border_radius(styles::BUBBLE_RADIUS)
            .background(PREVIEW_BACKGROUND)
            .font_size(styles::FONT_META)
            .color(styles::COLOR_TEXT)
    })
}

fn action_row<OnOutputAction>(
    output_id: u64,
    actions: Vec<HostActionIntent>,
    on_output_action: OnOutputAction,
) -> AnyView
where
    OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
{
    if actions.is_empty() {
        return empty().into_any();
    }
    h_stack_from_iter(
        actions
            .into_iter()
            .map(move |action| output_action_button(output_id, action.kind, on_output_action)),
    )
    .style(|style| style.gap(CARD_GAP).items_center().justify_end())
    .into_any()
}

fn output_action_button<OnOutputAction>(
    output_id: u64,
    action: HostActionKind,
    on_output_action: OnOutputAction,
) -> impl IntoView
where
    OnOutputAction: Fn(u64, HostActionKind) + Copy + 'static,
{
    button(label(move || action_label(action).to_string()))
        .action(move || on_output_action(output_id, action))
        .style(|style| {
            style
                .padding_horiz(10.0)
                .padding_vert(4.0)
                .border_radius(styles::BUBBLE_RADIUS)
                .font_size(styles::FONT_META)
                .color(styles::COLOR_TEXT)
                .background(Color::WHITE)
        })
}

fn action_label(action: HostActionKind) -> &'static str {
    match action {
        HostActionKind::Copy => "Copy",
        HostActionKind::OpenPreview => "Preview",
        HostActionKind::CreateFile => "Create",
        HostActionKind::ApplyDiff => "Apply",
        HostActionKind::UndoChange => "Undo",
        HostActionKind::Approve => "Approve",
        HostActionKind::Reject => "Reject",
    }
}

struct OutputCardSummary {
    title: String,
    detail: String,
}

impl OutputCardSummary {
    fn from_output(output: ChatUiOutputSurface) -> Self {
        match output.kind {
            ChatOutputKind::Text(text) => Self::new("テキスト出力", text.text),
            ChatOutputKind::Code(code) => {
                let label = code.language.unwrap_or_else(|| "code".to_string());
                Self::new(format!("コード出力: {label}"), code.code)
            }
            ChatOutputKind::FileCandidate(file) => Self::new(
                format!("ファイル候補: {}", file.path),
                format!("{}\n\n{}", file.mime_type, file.content),
            ),
            ChatOutputKind::DiffCandidate(diff) => {
                Self::new(format!("差分候補: {}", diff.target_path), diff.unified_diff)
            }
            ChatOutputKind::ToolResult(result) => {
                Self::new(format!("ツール結果: {}", result.tool_name), result.summary)
            }
            ChatOutputKind::PermissionRequest(request) => Self::new(
                format!("承認要求: {}", request.action_label),
                request.reason,
            ),
        }
    }

    fn new(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: preview(detail.into()),
        }
    }
}

fn preview(detail: String) -> String {
    let mut chars = detail.chars();
    let text = chars.by_ref().take(PREVIEW_LIMIT).collect::<String>();
    if chars.next().is_none() {
        return text;
    }
    format!("{text}\n...")
}

#[cfg(test)]
mod tests {
    use super::*;
    use katana_chat_ui::{
        ChatOutputKind, CodeOutput, DiffCandidateOutput, FileCandidateOutput, HostActionIntent,
        PermissionRequestOutput, TextOutput, ToolResultOutput,
    };

    #[test]
    fn text_output_summary_keeps_text_as_preview_body() {
        let summary =
            OutputCardSummary::from_output(output(ChatOutputKind::Text(TextOutput::new("hello"))));

        assert_eq!(summary.title, "テキスト出力");
        assert_eq!(summary.detail, "hello");
    }

    #[test]
    fn code_output_summary_uses_language_or_code_label() {
        let with_language = OutputCardSummary::from_output(output(ChatOutputKind::Code(
            CodeOutput::new(Some("rust".to_string()), "fn main() {}"),
        )));
        let without_language = OutputCardSummary::from_output(output(ChatOutputKind::Code(
            CodeOutput::new(None, "echo hi"),
        )));

        assert_eq!(with_language.title, "コード出力: rust");
        assert_eq!(without_language.title, "コード出力: code");
    }

    #[test]
    fn file_output_summary_keeps_path_mime_type_and_content() {
        let summary = OutputCardSummary::from_output(output(ChatOutputKind::FileCandidate(
            FileCandidateOutput::new("tmp/generated.md", "text/markdown", "# generated"),
        )));

        assert_eq!(summary.title, "ファイル候補: tmp/generated.md");
        assert!(summary.detail.contains("text/markdown"));
        assert!(summary.detail.contains("# generated"));
    }

    #[test]
    fn diff_tool_and_permission_outputs_use_action_specific_titles() {
        let diff = OutputCardSummary::from_output(output(ChatOutputKind::DiffCandidate(
            DiffCandidateOutput::new(
                "src/lib.rs",
                "before",
                "after",
                "@@ -1 +1 @@",
                "src/lib.rs を更新",
            ),
        )));
        let tool = OutputCardSummary::from_output(output(ChatOutputKind::ToolResult(
            ToolResultOutput::new("cargo test", "passed"),
        )));
        let permission = OutputCardSummary::from_output(output(ChatOutputKind::PermissionRequest(
            PermissionRequestOutput::new("apply patch", "needs approval"),
        )));

        assert_eq!(diff.title, "差分候補: src/lib.rs");
        assert_eq!(tool.title, "ツール結果: cargo test");
        assert_eq!(permission.title, "承認要求: apply patch");
    }

    #[test]
    fn preview_truncates_long_detail_without_byte_splitting() {
        let detail = "あ".repeat(PREVIEW_LIMIT + 2);

        assert!(preview(detail).ends_with("\n..."));
    }

    #[test]
    fn preview_leaves_short_detail_unchanged() {
        assert_eq!(preview("short output".to_string()), "short output");
    }

    fn output(kind: ChatOutputKind) -> ChatUiOutputSurface {
        ChatUiOutputSurface {
            id: 1,
            source_message_id: 2,
            kind,
            actions: Vec::<HostActionIntent>::new(),
        }
    }
}
