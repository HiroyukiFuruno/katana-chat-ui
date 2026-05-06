use crate::{ComposerView, MessageListView, OutputListView, UsageMeterView};
use katana_chat_ui::ChatRenderModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPanelView {
    pub composer: ComposerView,
    pub message_list: MessageListView,
    pub output_list: OutputListView,
    pub usage_meter: UsageMeterView,
    pub settings_trigger_visible: bool,
}

impl ChatPanelView {
    pub fn from_render_model(model: &ChatRenderModel) -> Self {
        Self {
            composer: ComposerView::from_model(&model.input),
            message_list: MessageListView::from_model(&model.messages),
            output_list: OutputListView::from_model(&model.outputs),
            usage_meter: UsageMeterView::from_model(&model.context_usage, &model.account_usage),
            settings_trigger_visible: true,
        }
    }
}

#[cfg(test)]
mod tests;
