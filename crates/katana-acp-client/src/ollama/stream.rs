use super::types::OllamaChatStreamResponse;
use crate::{AcpError, AiStreamEvent};

pub(super) struct OllamaStreamDecoder;

impl OllamaStreamDecoder {
    #[cfg(test)]
    pub(super) fn append<OnChunk>(
        pending: &mut String,
        chunk: &[u8],
        on_chunk: &mut OnChunk,
    ) -> Result<(), AcpError>
    where
        OnChunk: FnMut(String) -> Result<(), AcpError>,
    {
        Self::append_event(pending, chunk, &mut |event| {
            if let AiStreamEvent::Content(content) = event {
                return on_chunk(content);
            }
            Ok(())
        })
    }

    pub(super) fn append_event<OnEvent>(
        pending: &mut String,
        chunk: &[u8],
        on_event: &mut OnEvent,
    ) -> Result<(), AcpError>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), AcpError>,
    {
        let text =
            std::str::from_utf8(chunk).map_err(|error| AcpError::Protocol(error.to_string()))?;
        pending.push_str(text);
        while let Some(line_end) = pending.find('\n') {
            let line = pending[..line_end].trim().to_string();
            pending.drain(..=line_end);
            Self::process_line_event(&line, on_event)?;
        }
        Ok(())
    }

    pub(super) fn finish_event<OnEvent>(
        pending: &str,
        on_event: &mut OnEvent,
    ) -> Result<(), AcpError>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), AcpError>,
    {
        Self::process_line_event(pending.trim(), on_event)
    }

    fn process_line_event<OnEvent>(line: &str, on_event: &mut OnEvent) -> Result<(), AcpError>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), AcpError>,
    {
        if line.is_empty() {
            return Ok(());
        }
        let response: OllamaChatStreamResponse =
            serde_json::from_str(line).map_err(|error| AcpError::Protocol(error.to_string()))?;
        let Some(message) = response.message else {
            return Ok(());
        };
        if let Some(thinking) = message.thinking.filter(|it| !it.is_empty()) {
            on_event(AiStreamEvent::Thinking(thinking))?;
        }
        if !message.content.is_empty() {
            on_event(AiStreamEvent::Content(message.content))?;
        }
        Ok(())
    }
}
