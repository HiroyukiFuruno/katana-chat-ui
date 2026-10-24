use katana_chat_ui::{ChatOutput, ChatOutputKind, OutputStatus};

pub(crate) fn append_for_message(context: &mut String, message_id: u64, outputs: &[ChatOutput]) {
    for output in outputs {
        if output.source_message_id != message_id {
            continue;
        }
        context.push_str("\n<message-output id=\"");
        context.push_str(&output.id.to_string());
        context.push_str("\" kind=\"");
        context.push_str(kind_name(&output.kind));
        context.push_str("\" status=\"");
        append_status_name(context, &output.status);
        context.push_str("\">");
        append_kind(context, &output.kind);
        append_status_detail(context, &output.status);
        context.push_str("</message-output>");
    }
}

fn kind_name(kind: &ChatOutputKind) -> &'static str {
    match kind {
        ChatOutputKind::Text(_) => "text",
        ChatOutputKind::Code(_) => "code",
        ChatOutputKind::FileCandidate(_) => "file-candidate",
        ChatOutputKind::DiffCandidate(_) => "diff-candidate",
        ChatOutputKind::ToolResult(_) => "tool-result",
        ChatOutputKind::PermissionRequest(_) => "permission-request",
    }
}

fn append_status_name(context: &mut String, status: &OutputStatus) {
    context.push_str(match status {
        OutputStatus::Ready => "ready",
        OutputStatus::PendingHost => "pending-host",
        OutputStatus::Applied => "applied",
        OutputStatus::Reverted => "reverted",
        OutputStatus::Rejected => "rejected",
        OutputStatus::Failed(_) => "failed",
    });
}

fn append_status_detail(context: &mut String, status: &OutputStatus) {
    if let OutputStatus::Failed(reason) = status {
        element(context, "failure-reason", reason);
    }
}

fn append_kind(context: &mut String, kind: &ChatOutputKind) {
    match kind {
        ChatOutputKind::Text(value) => element(context, "text", &value.text),
        ChatOutputKind::Code(value) => {
            if let Some(language) = &value.language {
                element(context, "language", language);
            }
            element(context, "code", &value.code);
        }
        ChatOutputKind::FileCandidate(value) => {
            element(context, "path", &value.path);
            element(context, "mime-type", &value.mime_type);
            element(context, "saved-content", &value.content);
        }
        ChatOutputKind::DiffCandidate(value) => {
            element(context, "target-path", &value.target_path);
            element(context, "original-content", &value.original_content);
            element(context, "updated-content", &value.updated_content);
            element(context, "unified-diff", &value.unified_diff);
            element(context, "summary", &value.summary);
        }
        ChatOutputKind::ToolResult(value) => {
            element(context, "tool-name", &value.tool_name);
            element(context, "summary", &value.summary);
        }
        ChatOutputKind::PermissionRequest(value) => {
            element(context, "action-label", &value.action_label);
            element(context, "reason", &value.reason);
        }
    }
}

fn element(context: &mut String, name: &str, value: &str) {
    context.push('<');
    context.push_str(name);
    context.push('>');
    append_data(context, value);
    context.push_str("</");
    context.push_str(name);
    context.push('>');
}

fn append_data(context: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => context.push_str("&amp;"),
            '<' => context.push_str("&lt;"),
            '>' => context.push_str("&gt;"),
            _ => context.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use katana_chat_ui::{
        ChatOutput, ChatOutputKind, CodeOutput, DiffCandidateOutput, FileCandidateOutput,
        OutputStatus, PermissionRequestOutput, TextOutput, ToolResultOutput,
    };

    use super::append_for_message;

    fn output(id: u64, source: u64, kind: ChatOutputKind, status: OutputStatus) -> ChatOutput {
        let mut output = ChatOutput::new(id, source, kind);
        output.status = status;
        output
    }

    fn text_and_code_outputs() -> Vec<ChatOutput> {
        vec![
            output(
                1,
                7,
                ChatOutputKind::Text(TextOutput::new("text <&>")),
                OutputStatus::Ready,
            ),
            output(
                2,
                7,
                ChatOutputKind::Code(CodeOutput::new(Some("rust".into()), "fn main() {}")),
                OutputStatus::PendingHost,
            ),
        ]
    }

    fn candidate_outputs() -> Vec<ChatOutput> {
        vec![
            output(
                3,
                7,
                ChatOutputKind::FileCandidate(FileCandidateOutput::new(
                    "tmp/new.md",
                    "text/markdown",
                    "saved <content>",
                )),
                OutputStatus::Applied,
            ),
            output(
                4,
                7,
                ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
                    "tmp/edit.md",
                    "before",
                    "after & changed",
                    "@@ -1 +1 @@",
                    "summary",
                )),
                OutputStatus::Reverted,
            ),
        ]
    }

    fn metadata_outputs() -> Vec<ChatOutput> {
        vec![
            output(
                5,
                7,
                ChatOutputKind::ToolResult(ToolResultOutput::new("search", "tool summary")),
                OutputStatus::Rejected,
            ),
            output(
                6,
                7,
                ChatOutputKind::PermissionRequest(PermissionRequestOutput::new(
                    "write",
                    "reason </message>",
                )),
                OutputStatus::Failed("failure & reason".into()),
            ),
        ]
    }

    fn all_kinds() -> Vec<ChatOutput> {
        let mut outputs = text_and_code_outputs();
        outputs.extend(candidate_outputs());
        outputs.extend(metadata_outputs());
        outputs
    }

    #[test]
    fn filters_by_source_and_preserves_output_order() -> Result<(), String> {
        let mut outputs = all_kinds();
        outputs.insert(
            1,
            output(
                99,
                8,
                ChatOutputKind::Text(TextOutput::new("orphan")),
                OutputStatus::Ready,
            ),
        );
        let mut context = String::new();
        append_for_message(&mut context, 7, &outputs);
        assert!(!context.contains("orphan"));
        let first = context.find("id=\"1\"").ok_or("first output is missing")?;
        let last = context.find("id=\"6\"").ok_or("last output is missing")?;
        if first >= last {
            return Err("output order changed".into());
        }
        assert_eq!(context.matches("<message-output ").count(), 6);
        Ok(())
    }

    #[test]
    fn unmatched_outputs_leave_context_unchanged() {
        let outputs = vec![output(
            8,
            9,
            ChatOutputKind::Code(CodeOutput::new(None, "orphan code")),
            OutputStatus::Ready,
        )];
        let mut context = String::from("prefix");
        append_for_message(&mut context, 7, &outputs);
        assert_eq!(context, "prefix");
    }

    #[test]
    fn serializes_all_kinds_statuses_and_escaped_saved_data() {
        let mut context = String::new();
        append_for_message(&mut context, 7, &all_kinds());
        for status in [
            "ready",
            "pending-host",
            "applied",
            "reverted",
            "rejected",
            "failed",
        ] {
            assert!(context.contains(&format!("status=\"{status}\"")));
        }
        for kind in [
            "text",
            "code",
            "file-candidate",
            "diff-candidate",
            "tool-result",
            "permission-request",
        ] {
            assert!(context.contains(&format!("kind=\"{kind}\"")));
        }
        assert!(context.contains("<saved-content>saved &lt;content&gt;</saved-content>"));
        assert!(context.contains("<updated-content>after &amp; changed</updated-content>"));
        assert!(context.contains("<failure-reason>failure &amp; reason</failure-reason>"));
        assert!(!context.contains("reason </message>"));
    }
}
