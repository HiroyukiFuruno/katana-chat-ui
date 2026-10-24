use super::{ChatSession, ChatSessionError};
use crate::{
    AgentActivity, AgentActivityKind, ChatMessage, MessageRole, MessageStatus, ThinkingLog,
};

impl ChatSession {
    pub fn start_assistant_stream(
        &mut self,
        initial_content: impl Into<String>,
    ) -> Result<u64, ChatSessionError> {
        self.push_assistant_stream(initial_content, None)
    }

    pub fn start_assistant_stream_with_thinking(
        &mut self,
        initial_content: impl Into<String>,
        thinking: ThinkingLog,
    ) -> Result<u64, ChatSessionError> {
        self.push_assistant_stream(initial_content, Some(thinking))
    }

    pub fn append_assistant_chunk(
        &mut self,
        chunk: impl AsRef<str>,
    ) -> Result<(), ChatSessionError> {
        let message = self.latest_streaming_assistant_mut()?;
        message.append_content(chunk);
        Ok(())
    }

    pub fn append_assistant_thinking(
        &mut self,
        label: impl Into<String>,
        chunk: impl AsRef<str>,
    ) -> Result<(), ChatSessionError> {
        let message = self.latest_streaming_assistant_mut()?;
        message.append_thinking(label, chunk);
        Ok(())
    }

    pub fn set_assistant_activity(
        &mut self,
        kind: AgentActivityKind,
    ) -> Result<(), ChatSessionError> {
        let message = self.latest_streaming_assistant_mut()?;
        message.set_activity(AgentActivity::new(kind));
        Ok(())
    }

    pub fn finish_assistant_message(&mut self) -> Result<(), ChatSessionError> {
        let message = self.latest_streaming_assistant_mut()?;
        message.finish_thinking();
        message.clear_activity();
        message.set_status(MessageStatus::Complete);
        Ok(())
    }

    pub fn fail_assistant_message(
        &mut self,
        reason: impl Into<String>,
    ) -> Result<(), ChatSessionError> {
        let message = self.latest_streaming_assistant_mut()?;
        message.finish_thinking();
        message.clear_activity();
        message.set_status(MessageStatus::Error(reason.into()));
        Ok(())
    }

    pub(crate) fn has_streaming_message(&self) -> bool {
        self.messages
            .iter()
            .any(|it| it.status == MessageStatus::Streaming)
    }

    fn push_assistant_stream(
        &mut self,
        initial_content: impl Into<String>,
        thinking: Option<ThinkingLog>,
    ) -> Result<u64, ChatSessionError> {
        self.ensure_provider_configured()?;
        let message_id = self.next_message_id();
        let message = self.stream_message(message_id, initial_content, thinking);
        self.messages.push(message);
        Ok(message_id)
    }

    fn stream_message(
        &self,
        message_id: u64,
        initial_content: impl Into<String>,
        thinking: Option<ThinkingLog>,
    ) -> ChatMessage {
        let message = ChatMessage::new(message_id, MessageRole::Assistant, initial_content)
            .with_status(MessageStatus::Streaming);
        match thinking {
            Some(thinking) => message.with_thinking(thinking),
            None => message.with_activity(AgentActivity::processing()),
        }
    }

    fn latest_streaming_assistant_mut(&mut self) -> Result<&mut ChatMessage, ChatSessionError> {
        for message in self.messages.iter_mut().rev() {
            let is_target = message.role == MessageRole::Assistant
                && message.status == MessageStatus::Streaming;
            if is_target {
                return Ok(message);
            }
        }
        Err(ChatSessionError::StreamingMessageNotFound)
    }
}
