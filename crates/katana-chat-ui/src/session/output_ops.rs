use super::{ChatSession, ChatSessionError};
use crate::{ChatOutput, ChatOutputKind, OutputStatus};

impl ChatSession {
    pub fn add_output(
        &mut self,
        source_message_id: u64,
        kind: ChatOutputKind,
    ) -> Result<u64, ChatSessionError> {
        self.ensure_message_exists(source_message_id)?;
        let output_id = self.next_output_id();
        self.outputs
            .push(ChatOutput::new(output_id, source_message_id, kind));
        Ok(output_id)
    }

    pub fn set_output_status(
        &mut self,
        output_id: u64,
        status: OutputStatus,
    ) -> Result<(), ChatSessionError> {
        let output = self.output_mut(output_id)?;
        output.status = status;
        Ok(())
    }

    fn ensure_message_exists(&self, message_id: u64) -> Result<(), ChatSessionError> {
        if self.messages.iter().any(|it| it.id == message_id) {
            return Ok(());
        }
        Err(ChatSessionError::MessageNotFound)
    }

    fn output_mut(&mut self, output_id: u64) -> Result<&mut ChatOutput, ChatSessionError> {
        for output in &mut self.outputs {
            if output.id == output_id {
                return Ok(output);
            }
        }
        Err(ChatSessionError::OutputNotFound)
    }

    fn next_output_id(&mut self) -> u64 {
        let output_id = self.next_output_id;
        self.next_output_id = self.next_output_id.saturating_add(1);
        output_id
    }
}
