#[derive(Clone, Copy)]
pub struct FloemChatActions<
    OnAttach,
    OnRemoveAttachment,
    OnNewChat,
    OnHistory,
    OnOutputAction,
    OnSubmit,
    OnStop,
    OnVendorSelect,
    OnControlSelect,
> {
    pub on_attach: OnAttach,
    pub on_remove_attachment: OnRemoveAttachment,
    pub on_new_chat: OnNewChat,
    pub on_history: OnHistory,
    pub on_output_action: OnOutputAction,
    pub on_submit: OnSubmit,
    pub on_stop: OnStop,
    pub on_vendor_select: OnVendorSelect,
    pub on_control_select: OnControlSelect,
}
