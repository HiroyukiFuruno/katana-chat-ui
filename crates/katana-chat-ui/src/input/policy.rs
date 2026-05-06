use super::{Attachment, UnsupportedAttachment};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentPolicy {
    images_supported: bool,
    files_supported: bool,
    unsupported_image_reason: String,
    unsupported_file_reason: String,
}

impl AttachmentPolicy {
    pub fn all_supported() -> Self {
        Self {
            images_supported: true,
            files_supported: true,
            unsupported_image_reason: String::new(),
            unsupported_file_reason: String::new(),
        }
    }

    pub fn text_and_files_only(image_reason: impl Into<String>) -> Self {
        Self {
            images_supported: false,
            files_supported: true,
            unsupported_image_reason: image_reason.into(),
            unsupported_file_reason: String::new(),
        }
    }

    pub fn text_only(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            images_supported: false,
            files_supported: false,
            unsupported_image_reason: reason.clone(),
            unsupported_file_reason: reason,
        }
    }

    pub(crate) fn validate(&self, attachment: &Attachment, validation: &mut AttachmentValidation) {
        match attachment {
            Attachment::ImageResource(image) if !self.images_supported => {
                validation.push(
                    image.data_ref.clone(),
                    self.unsupported_image_reason.clone(),
                );
            }
            Attachment::FileResource(file) if !self.files_supported => {
                validation.push(file.uri.clone(), self.unsupported_file_reason.clone());
            }
            Attachment::Unsupported(unsupported) => {
                validation.push(unsupported.label.clone(), unsupported.reason.clone());
            }
            _ => {}
        }
    }
}

impl Default for AttachmentPolicy {
    fn default() -> Self {
        Self::all_supported()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentValidation {
    unsupported: Vec<UnsupportedAttachment>,
}

impl AttachmentValidation {
    pub fn new() -> Self {
        Self {
            unsupported: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.unsupported.is_empty()
    }

    pub fn unsupported_reasons(&self) -> Vec<&str> {
        self.unsupported
            .iter()
            .map(|it| it.reason.as_str())
            .collect()
    }

    fn push(&mut self, label: String, reason: String) {
        self.unsupported
            .push(UnsupportedAttachment::new(label, reason));
    }
}

impl Default for AttachmentValidation {
    fn default() -> Self {
        Self::new()
    }
}
