use super::{ChatSession, ChatSessionError};
use crate::{ChatAgentEvent, ChatOutputKind, MessageRole, MessageStatus};

impl ChatSession {
    pub fn apply_agent_event(&mut self, event: ChatAgentEvent) -> Result<(), ChatSessionError> {
        match event {
            ChatAgentEvent::Chunk { content } => self.append_assistant_chunk(content),
            ChatAgentEvent::Activity { kind } => self.set_assistant_activity(kind),
            ChatAgentEvent::ThinkingChunk { label, content } => {
                self.append_assistant_thinking(label, content)
            }
            ChatAgentEvent::Output { kind } => self.add_agent_output(kind),
            ChatAgentEvent::Complete => self.finish_assistant_message(),
            ChatAgentEvent::Failed { reason } => self.fail_assistant_message(reason),
        }
    }

    fn add_agent_output(&mut self, kind: ChatOutputKind) -> Result<(), ChatSessionError> {
        let source_message_id = self.latest_streaming_assistant_id()?;
        self.add_output(source_message_id, kind)?;
        Ok(())
    }

    fn latest_streaming_assistant_id(&self) -> Result<u64, ChatSessionError> {
        for message in self.messages.iter().rev() {
            let is_streaming_assistant = message.role == MessageRole::Assistant
                && message.status == MessageStatus::Streaming;
            if is_streaming_assistant {
                return Ok(message.id);
            }
        }
        Err(ChatSessionError::StreamingMessageNotFound)
    }
}
