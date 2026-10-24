use katana_acp_client::{AcpContentBlock, AcpEmbeddedResource, AcpImageContent, AcpTextContent};
use serde::{Deserialize, Serialize};

use super::UnsupportedAttachment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attachment {
    Text(TextAttachment),
    FileResource(FileResource),
    ImageResource(ImageResource),
    Unsupported(UnsupportedAttachment),
}

impl Attachment {
    pub fn text(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self::Text(TextAttachment::new(label, text))
    }

    pub fn file(resource: FileResource) -> Self {
        Self::FileResource(resource)
    }

    pub fn image(resource: ImageResource) -> Self {
        Self::ImageResource(resource)
    }

    pub fn unsupported(label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Unsupported(UnsupportedAttachment::new(label, reason))
    }

    pub(crate) fn push_acp_block(&self, blocks: &mut Vec<AcpContentBlock>) {
        match self {
            Self::Text(text) => blocks.push(text.acp_block()),
            Self::FileResource(file) => blocks.push(file.acp_block()),
            Self::ImageResource(image) => blocks.push(image.acp_block()),
            Self::Unsupported(_) => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextAttachment {
    pub label: String,
    pub text: String,
}

impl TextAttachment {
    pub fn new(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            text: text.into(),
        }
    }

    fn acp_block(&self) -> AcpContentBlock {
        AcpContentBlock::Text(AcpTextContent::new(self.text.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileResource {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
    pub size_bytes: u64,
}

impl FileResource {
    pub fn new(
        uri: impl Into<String>,
        mime_type: impl Into<String>,
        text: impl Into<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            uri: uri.into(),
            mime_type: mime_type.into(),
            text: text.into(),
            size_bytes,
        }
    }

    fn acp_block(&self) -> AcpContentBlock {
        AcpContentBlock::EmbeddedResource(AcpEmbeddedResource::new(
            self.uri.clone(),
            self.mime_type.clone(),
            self.text.clone(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageResource {
    pub mime_type: String,
    pub data_ref: String,
    pub size_bytes: u64,
}

impl ImageResource {
    pub fn new(mime_type: impl Into<String>, data_ref: impl Into<String>, size_bytes: u64) -> Self {
        Self {
            mime_type: mime_type.into(),
            data_ref: data_ref.into(),
            size_bytes,
        }
    }

    fn acp_block(&self) -> AcpContentBlock {
        AcpContentBlock::Image(AcpImageContent::new(
            self.mime_type.clone(),
            self.data_ref.clone(),
        ))
    }
}
