use katana_chat_ui::{ChatOutputKind, HostActionKind, OutputRenderModel, OutputStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputListView {
    pub outputs: Vec<OutputRowView>,
}

impl OutputListView {
    pub fn from_model(outputs: &[OutputRenderModel]) -> Self {
        Self {
            outputs: outputs.iter().map(OutputRowView::from_model).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputRowView {
    pub source_message_id: u64,
    pub kind: ChatOutputKind,
    pub status: OutputStatus,
    pub actions: Vec<HostActionKind>,
}

impl OutputRowView {
    fn from_model(output: &OutputRenderModel) -> Self {
        Self {
            source_message_id: output.source_message_id,
            kind: output.kind.clone(),
            status: output.status.clone(),
            actions: output.actions.iter().map(|it| it.kind).collect(),
        }
    }
}
