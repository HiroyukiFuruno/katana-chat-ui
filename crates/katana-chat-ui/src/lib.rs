//! Framework-neutral AI chat state and render model.

pub mod config;
pub mod input;
pub mod markdown;
pub mod message;
pub mod output;
pub mod render_model;
pub mod session;
pub mod surface;
pub mod text;
pub mod theme;
pub mod usage;
pub mod vendor_ui;

pub use config::ChatUiConfig;
pub use input::{
    Attachment, AttachmentPolicy, ChatInputDraft, FileResource, ImageResource, PathDropRequest,
};
pub use markdown::{
    CodeBlock, HeadingBlock, InlineSegment, ListItem, ListKind, MarkdownBlock, MarkdownSubset,
    TableBlock, TextBlock,
};
pub use message::{
    ChatMessage, MessageAlignment, MessageRole, MessageStatus, RoleVisualIntent, ThinkingLog,
};
pub use output::{
    ChatOutput, ChatOutputKind, CodeOutput, DiffCandidateOutput, FileCandidateOutput,
    HostActionIntent, HostActionKind, OutputStatus, PermissionRequestOutput, TextOutput,
    ToolResultOutput,
};
pub use render_model::{
    ChatIconSet, ChatRenderModel, ChatUiOptions, InputRenderModel, MessageRenderModel,
    OutputRenderModel, ProviderConnectionState, ThinkingRenderModel,
};
pub use session::{ChatSession, ChatSessionError};
pub use surface::{
    ChatUiActionButtonSurface, ChatUiChromeSurface, ChatUiComposerInputKind, ChatUiComposerSurface,
    ChatUiDebugSurface, ChatUiMessageAlignment, ChatUiMessageListSurface, ChatUiMessageSurface,
    ChatUiOutputHandoffSurface, ChatUiOutputSurface, ChatUiSurface, ChatUiSurfaceProvider,
    ChatUiThinkingSurface, ChatUiUsageSurface, ChatUiVendorBarSurface, ChatUiVendorControlSurface,
};
pub use text::{
    ChatLocale, ChatTextKey, ChatTextSet, TextCatalog, TextCatalogError, TextCatalogOverride,
};
pub use theme::{IconRegistry, SvgIcon, ThemeOverride, ThemeTokens};
pub use usage::{AccountUsageSnapshot, ContextUsageSnapshot, UsageStatus};
pub use vendor_ui::{
    OfficialReference, VendorCapabilityFact, VendorCapabilityStatus, VendorConnectionKind,
    VendorControlItem, VendorControlProvider, VendorControlRenderModel, VendorFact,
    VendorFactRegistry, VendorFactValidationError, VendorOption, VendorUiCapabilities,
    VendorUiProfile, VendorUiState, VendorUiSurface,
};
