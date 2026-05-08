use crate::widget::composer_controls::FloemComposerControlActions;

#[derive(Clone)]
pub struct FloemComposerActions<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect> {
    pub(super) on_attach: OnAttach,
    pub(super) on_remove_attachment: OnRemoveAttachment,
    pub(super) on_submit: OnSubmit,
    pub(super) on_stop: OnStop,
    pub(super) on_control_select: OnControlSelect,
}

impl<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>
    FloemComposerActions<OnAttach, OnRemoveAttachment, OnSubmit, OnStop, OnControlSelect>
{
    pub fn new(
        on_attach: OnAttach,
        on_remove_attachment: OnRemoveAttachment,
        on_submit: OnSubmit,
        on_stop: OnStop,
        on_control_select: OnControlSelect,
    ) -> Self {
        Self {
            on_attach,
            on_remove_attachment,
            on_submit,
            on_stop,
            on_control_select,
        }
    }

    pub(super) fn control_actions(
        &self,
    ) -> FloemComposerControlActions<OnAttach, OnStop, OnSubmit, OnControlSelect>
    where
        OnAttach: Copy,
        OnSubmit: Clone,
        OnStop: Copy,
        OnControlSelect: Copy,
    {
        FloemComposerControlActions::new(
            self.on_attach,
            self.on_stop,
            self.on_submit.clone(),
            self.on_control_select,
        )
    }
}
