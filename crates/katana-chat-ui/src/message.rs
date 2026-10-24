use crate::Attachment;
use serde::{Deserialize, Serialize};

pub type MessageId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: String,
    pub status: MessageStatus,
    #[serde(default)]
    pub activity: Option<AgentActivity>,
    pub thinking: Option<ThinkingLog>,
    pub attachments: Vec<Attachment>,
}

impl ChatMessage {
    pub fn new(id: MessageId, role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            id,
            role,
            content: content.into(),
            status: MessageStatus::Complete,
            activity: None,
            thinking: None,
            attachments: Vec::new(),
        }
    }

    pub fn with_status(mut self, status: MessageStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_activity(mut self, activity: AgentActivity) -> Self {
        self.activity = Some(activity);
        self
    }

    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = attachments;
        self
    }

    pub fn with_thinking(mut self, thinking: ThinkingLog) -> Self {
        self.thinking = Some(thinking);
        self
    }

    pub fn append_content(&mut self, content: impl AsRef<str>) {
        self.content.push_str(content.as_ref());
    }

    pub fn set_status(&mut self, status: MessageStatus) {
        self.status = status;
    }

    pub fn set_activity(&mut self, activity: AgentActivity) {
        self.activity = Some(activity);
    }

    pub fn clear_activity(&mut self) {
        self.activity = None;
    }

    pub fn finish_thinking(&mut self) {
        if let Some(thinking) = self.thinking.as_mut() {
            thinking.completed = true;
            thinking.expanded = false;
        }
    }

    pub fn append_thinking(&mut self, label: impl Into<String>, chunk: impl AsRef<str>) {
        match self.thinking.as_mut() {
            Some(thinking) => thinking.append_chunk(chunk),
            None => {
                self.thinking = Some(ThinkingLog::running(
                    label,
                    vec![chunk.as_ref().to_string()],
                ));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    System,
}

impl MessageRole {
    pub fn visual_intent(&self) -> RoleVisualIntent {
        match self {
            Self::User => RoleVisualIntent::new(MessageAlignment::Trailing, "user-bubble"),
            Self::Assistant => RoleVisualIntent::new(MessageAlignment::Leading, "assistant-bubble"),
            Self::Tool => RoleVisualIntent::new(MessageAlignment::FullWidth, "tool-surface"),
            Self::System => RoleVisualIntent::new(MessageAlignment::FullWidth, "system-note"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Sending,
    Streaming,
    Complete,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentActivity {
    pub kind: AgentActivityKind,
}

impl AgentActivity {
    pub fn new(kind: AgentActivityKind) -> Self {
        Self { kind }
    }

    pub fn processing() -> Self {
        Self::new(AgentActivityKind::Processing)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentActivityKind {
    Processing,
    Generating,
    Editing,
    Reading,
    Searching,
    WebSearching,
    Executing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingLog {
    pub label: String,
    pub entries: Vec<String>,
    pub expanded: bool,
    pub completed: bool,
}

impl ThinkingLog {
    pub fn running(label: impl Into<String>, entries: Vec<String>) -> Self {
        Self {
            label: label.into(),
            entries,
            expanded: true,
            completed: false,
        }
    }

    pub fn append_chunk(&mut self, chunk: impl AsRef<str>) {
        let chunk = chunk.as_ref();
        if chunk.is_empty() {
            return;
        }
        match self.entries.last_mut() {
            Some(entry) => entry.push_str(chunk),
            None => self.entries.push(chunk.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageAlignment {
    Leading,
    Trailing,
    FullWidth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleVisualIntent {
    pub alignment: MessageAlignment,
    pub surface_token: String,
}

impl RoleVisualIntent {
    pub fn new(alignment: MessageAlignment, surface_token: impl Into<String>) -> Self {
        Self {
            alignment,
            surface_token: surface_token.into(),
        }
    }
}
