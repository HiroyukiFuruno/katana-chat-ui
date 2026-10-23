use super::types::OllamaChatStreamResponse;
use crate::{AcpError, AiStreamEvent};

pub(super) struct OllamaStreamDecoder;

impl OllamaStreamDecoder {
    #[cfg(test)]
    pub(super) fn append<OnChunk>(
        pending: &mut Vec<u8>,
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
        pending: &mut Vec<u8>,
        chunk: &[u8],
        on_event: &mut OnEvent,
    ) -> Result<(), AcpError>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), AcpError>,
    {
        pending.extend_from_slice(chunk);
        while let Some(line_end) = pending.iter().position(|byte| *byte == b'\n') {
            let line = pending.drain(..=line_end).collect::<Vec<_>>();
            let line = std::str::from_utf8(&line)
                .map_err(|error| AcpError::Protocol(error.to_string()))?;
            Self::process_line_event(line.trim(), on_event)?;
        }
        Ok(())
    }

    pub(super) fn finish_event<OnEvent>(
        pending: &[u8],
        on_event: &mut OnEvent,
    ) -> Result<(), AcpError>
    where
        OnEvent: FnMut(AiStreamEvent) -> Result<(), AcpError>,
    {
        if pending.is_empty() {
            return Ok(());
        }
        let line =
            std::str::from_utf8(pending).map_err(|error| AcpError::Protocol(error.to_string()))?;
        Self::process_line_event(line.trim(), on_event)
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
        if let Some(error) = response.error {
            return Err(AcpError::Protocol(error));
        }
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
