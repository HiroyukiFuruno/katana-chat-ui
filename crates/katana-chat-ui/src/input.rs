mod attachment;
mod path_drop;
mod policy;

use katana_acp_client::{AcpContentBlock, AcpTextContent};
use serde::{Deserialize, Serialize};

pub use attachment::{Attachment, FileResource, ImageResource, TextAttachment};
pub use path_drop::{PathDropRequest, UnsupportedAttachment};
pub use policy::{AttachmentPolicy, AttachmentValidation};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatInputDraft {
    pub text: String,
    pub attachments: Vec<Attachment>,
    pub intent: ChatInputIntent,
}

impl ChatInputDraft {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn add_attachment(&mut self, attachment: Attachment) {
        self.attachments.push(attachment);
    }

    pub fn remove_attachment(&mut self, index: usize) -> Option<Attachment> {
        if index >= self.attachments.len() {
            return None;
        }
        Some(self.attachments.remove(index))
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.attachments.clear();
        self.intent = ChatInputIntent::Idle;
    }

    pub fn can_submit(&self) -> bool {
        !self.text.trim().is_empty() || !self.attachments.is_empty()
    }

    pub fn take_attachments(&mut self) -> Vec<Attachment> {
        std::mem::take(&mut self.attachments)
    }

    pub fn to_acp_content_blocks(&self) -> Vec<AcpContentBlock> {
        let mut blocks = Vec::new();
        if !self.text.trim().is_empty() {
            blocks.push(AcpContentBlock::Text(AcpTextContent::new(
                self.text.clone(),
            )));
        }
        for attachment in &self.attachments {
            attachment.push_acp_block(&mut blocks);
        }
        blocks
    }

    pub fn request_path_drop(
        path: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> PathDropRequest {
        PathDropRequest::new(Self::file_uri(path.into()), mime_type)
    }

    pub fn validate_attachments(&self, policy: &AttachmentPolicy) -> AttachmentValidation {
        let mut validation = AttachmentValidation::new();
        for attachment in &self.attachments {
            policy.validate(attachment, &mut validation);
        }
        validation
    }

    fn file_uri(path: String) -> String {
        if path.starts_with("file://") {
            return path;
        }
        format!("file://{path}")
    }
}

impl Default for ChatInputDraft {
    fn default() -> Self {
        Self {
            text: String::new(),
            attachments: Vec::new(),
            intent: ChatInputIntent::Idle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatInputIntent {
    Idle,
    Submit,
    Cancel,
}

#[cfg(test)]
mod tests;
