use super::TokenName;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThemeOverride {
    pub(super) colors: ThemeColorOverride,
    pub(super) spacing: SpacingOverride,
    pub(super) typography: TypographyOverride,
}

impl ThemeOverride {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_accent(mut self, token: impl Into<String>) -> Self {
        self.colors.accent = Some(TokenName::new(token));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct ThemeColorOverride {
    pub(super) background: Option<TokenName>,
    pub(super) surface: Option<TokenName>,
    pub(super) user_bubble: Option<TokenName>,
    pub(super) assistant_bubble: Option<TokenName>,
    pub(super) border: Option<TokenName>,
    pub(super) muted_text: Option<TokenName>,
    pub(super) accent: Option<TokenName>,
    pub(super) danger: Option<TokenName>,
    pub(super) warning: Option<TokenName>,
    pub(super) success: Option<TokenName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct SpacingOverride {
    pub(super) compact: Option<u16>,
    pub(super) regular: Option<u16>,
    pub(super) spacious: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct TypographyOverride {
    pub(super) body: Option<String>,
    pub(super) code: Option<String>,
    pub(super) label: Option<String>,
}
