use crate::{AcpContentBlock, AcpError, AcpImageContent};
use agent_client_protocol::schema::v1::{
    ContentBlock, EmbeddedResource, EmbeddedResourceResource, ImageContent, TextContent,
    TextResourceContents,
};
use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AcpPromptCapabilities {
    pub image: bool,
    pub embedded_context: bool,
}

pub struct AcpContentEncoder;

impl AcpContentEncoder {
    pub fn encode(
        blocks: &[AcpContentBlock],
        capabilities: AcpPromptCapabilities,
    ) -> Result<Vec<ContentBlock>, AcpError> {
        blocks
            .iter()
            .map(|block| encode_block(block, capabilities))
            .collect()
    }
}

fn encode_block(
    block: &AcpContentBlock,
    capabilities: AcpPromptCapabilities,
) -> Result<ContentBlock, AcpError> {
    match block {
        AcpContentBlock::Text(content) => Ok(ContentBlock::Text(TextContent::new(&content.text))),
        AcpContentBlock::EmbeddedResource(content) if capabilities.embedded_context => {
            let resource = TextResourceContents::new(&content.text, &content.uri)
                .mime_type(content.mime_type.clone());
            Ok(ContentBlock::Resource(EmbeddedResource::new(
                EmbeddedResourceResource::TextResourceContents(resource),
            )))
        }
        AcpContentBlock::EmbeddedResource(_) => Err(unsupported("embeddedContext")),
        AcpContentBlock::Image(content) if capabilities.image => image(content),
        AcpContentBlock::Image(_) => Err(unsupported("image")),
    }
}

fn image(content: &AcpImageContent) -> Result<ContentBlock, AcpError> {
    let prefix = format!("data:{};base64,", content.mime_type);
    let data = content.data_ref.strip_prefix(&prefix).ok_or_else(|| {
        AcpError::Protocol("image content requires a matching MIME base64 data URI".to_string())
    })?;
    STANDARD
        .decode(data)
        .map_err(|error| AcpError::Protocol(error.to_string()))?;
    Ok(ContentBlock::Image(ImageContent::new(
        data,
        &content.mime_type,
    )))
}

fn unsupported(capability: &str) -> AcpError {
    AcpError::Protocol(format!(
        "agent did not advertise prompt capability: {capability}"
    ))
}

#[cfg(test)]
mod tests;
