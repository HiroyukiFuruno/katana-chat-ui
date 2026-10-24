use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedAttachment {
    pub label: String,
    pub reason: String,
}

impl UnsupportedAttachment {
    pub fn new(label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathDropRequest {
    pub uri: String,
    pub mime_type: String,
}

impl PathDropRequest {
    pub fn new(uri: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: mime_type.into(),
        }
    }
}
