use katana_chat_ui::{CommandLaunchKind, HostActionKind, kuc::ChatUiAction};

pub trait EguiChatController {
    fn composer_text_mut(&mut self) -> &mut String;
    fn attach(&mut self);
    fn submit(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn new_chat(&mut self) {}
    fn history(&mut self) {}
    fn select_vendor(&mut self, _vendor_id: String) {}
    fn select_control(&mut self, _key: String, _value: String) {}

    fn remove_attachment(&mut self, _index: usize) -> Result<(), String> {
        Err("host does not support removing attachments".into())
    }

    fn select_history(&mut self, _session_id: String) -> Result<(), String> {
        Err("host does not support restoring history".into())
    }

    fn select_slash_command(
        &mut self,
        _entry_id: String,
        _kind: CommandLaunchKind,
    ) -> Result<(), String> {
        Err("host does not support slash commands".into())
    }

    fn output(&mut self, _output_id: u64, _kind: HostActionKind) -> Result<(), String> {
        Err("host does not support output actions".into())
    }

    fn respond_permission(
        &mut self,
        _request_id: u64,
        _option_id: Option<String>,
    ) -> Result<(), String> {
        Err("host does not support permission responses".into())
    }

    fn handle_action(&mut self, action: ChatUiAction) -> Result<(), String> {
        match action {
            ChatUiAction::Attach => self.attach(),
            ChatUiAction::NewChat => self.new_chat(),
            ChatUiAction::ToggleHistory => self.history(),
            ChatUiAction::Submit => return self.submit(),
            ChatUiAction::Stop => return self.stop(),
            ChatUiAction::SelectVendor { vendor_id } => self.select_vendor(vendor_id),
            ChatUiAction::SelectVendorControl { key, value } => self.select_control(key, value),
            ChatUiAction::RemoveAttachment { index } => return self.remove_attachment(index),
            ChatUiAction::SelectHistory { session_id } => return self.select_history(session_id),
            ChatUiAction::SelectSlashCommand { entry_id, kind } => {
                return self.select_slash_command(entry_id, kind);
            }
            ChatUiAction::Output { output_id, kind } => return self.output(output_id, kind),
            ChatUiAction::RespondPermission {
                request_id,
                option_id,
            } => return self.respond_permission(request_id, option_id),
        }
        Ok(())
    }
}
