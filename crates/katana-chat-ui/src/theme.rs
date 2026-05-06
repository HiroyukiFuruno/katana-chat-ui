mod icons;
mod overrides;

use serde::{Deserialize, Serialize};

pub use icons::{IconRegistry, SvgIcon};
pub use overrides::ThemeOverride;

const COMPACT_SPACING: u16 = 4;
const REGULAR_SPACING: u16 = 8;
const SPACIOUS_SPACING: u16 = 12;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeTokens {
    pub colors: ThemeColors,
    pub spacing: SpacingTokens,
    pub typography: TypographyTokens,
}

impl ThemeTokens {
    pub fn merge(&self, override_tokens: ThemeOverride) -> Self {
        Self {
            colors: self.colors.merge(override_tokens.colors),
            spacing: self.spacing.merge(override_tokens.spacing),
            typography: self.typography.merge(override_tokens.typography),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: TokenName,
    pub surface: TokenName,
    pub user_bubble: TokenName,
    pub assistant_bubble: TokenName,
    pub border: TokenName,
    pub muted_text: TokenName,
    pub accent: TokenName,
    pub danger: TokenName,
    pub warning: TokenName,
    pub success: TokenName,
}

impl ThemeColors {
    fn merge(&self, override_tokens: overrides::ThemeColorOverride) -> Self {
        Self {
            background: Self::select(override_tokens.background, &self.background),
            surface: Self::select(override_tokens.surface, &self.surface),
            user_bubble: Self::select(override_tokens.user_bubble, &self.user_bubble),
            assistant_bubble: Self::select(
                override_tokens.assistant_bubble,
                &self.assistant_bubble,
            ),
            border: Self::select(override_tokens.border, &self.border),
            muted_text: Self::select(override_tokens.muted_text, &self.muted_text),
            accent: Self::select(override_tokens.accent, &self.accent),
            danger: Self::select(override_tokens.danger, &self.danger),
            warning: Self::select(override_tokens.warning, &self.warning),
            success: Self::select(override_tokens.success, &self.success),
        }
    }

    fn select(override_token: Option<TokenName>, default_token: &TokenName) -> TokenName {
        override_token.unwrap_or_else(|| default_token.clone())
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: TokenName::new("background"),
            surface: TokenName::new("surface"),
            user_bubble: TokenName::new("user-bubble"),
            assistant_bubble: TokenName::new("assistant-bubble"),
            border: TokenName::new("border"),
            muted_text: TokenName::new("muted-text"),
            accent: TokenName::new("accent"),
            danger: TokenName::new("danger"),
            warning: TokenName::new("warning"),
            success: TokenName::new("success"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenName {
    pub name: String,
}

impl TokenName {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpacingTokens {
    pub compact: u16,
    pub regular: u16,
    pub spacious: u16,
}

impl SpacingTokens {
    fn merge(&self, override_tokens: overrides::SpacingOverride) -> Self {
        Self {
            compact: override_tokens.compact.unwrap_or(self.compact),
            regular: override_tokens.regular.unwrap_or(self.regular),
            spacious: override_tokens.spacious.unwrap_or(self.spacious),
        }
    }
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            compact: COMPACT_SPACING,
            regular: REGULAR_SPACING,
            spacious: SPACIOUS_SPACING,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypographyTokens {
    pub body: String,
    pub code: String,
    pub label: String,
}

impl TypographyTokens {
    fn merge(&self, override_tokens: overrides::TypographyOverride) -> Self {
        Self {
            body: override_tokens.body.unwrap_or_else(|| self.body.clone()),
            code: override_tokens.code.unwrap_or_else(|| self.code.clone()),
            label: override_tokens.label.unwrap_or_else(|| self.label.clone()),
        }
    }
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            body: "body".to_string(),
            code: "code".to_string(),
            label: "label".to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
