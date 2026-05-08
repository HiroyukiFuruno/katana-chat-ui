mod locale_values;
mod locales;
mod overrides;
mod set;

use serde::{Deserialize, Serialize};

pub use overrides::TextCatalogOverride;
pub use set::ChatTextSet;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TextCatalogError {
    #[error("unsupported locale: {0}")]
    UnsupportedLocale(String),
    #[error("invalid text override JSON: {0}")]
    InvalidOverrideJson(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatLocale {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "zh-TW")]
    ZhTw,
    #[serde(rename = "ko")]
    Ko,
    #[serde(rename = "pt")]
    Pt,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "de")]
    De,
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "it")]
    It,
}

pub const SUPPORTED_LOCALE_COUNT: usize = 10;

impl ChatLocale {
    pub const SUPPORTED: [Self; SUPPORTED_LOCALE_COUNT] = [
        Self::En,
        Self::Ja,
        Self::ZhCn,
        Self::ZhTw,
        Self::Ko,
        Self::Pt,
        Self::Fr,
        Self::De,
        Self::Es,
        Self::It,
    ];

    pub fn from_code(code: &str) -> Result<Self, TextCatalogError> {
        let normalized = Self::normalize_code(code);
        match normalized.as_str() {
            "en" => Ok(Self::En),
            "ja" => Ok(Self::Ja),
            "zh-cn" => Ok(Self::ZhCn),
            "zh-tw" => Ok(Self::ZhTw),
            "ko" => Ok(Self::Ko),
            "pt" => Ok(Self::Pt),
            "fr" => Ok(Self::Fr),
            "de" => Ok(Self::De),
            "es" => Ok(Self::Es),
            "it" => Ok(Self::It),
            _ => Err(TextCatalogError::UnsupportedLocale(code.to_string())),
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ja => "ja",
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
            Self::Ko => "ko",
            Self::Pt => "pt",
            Self::Fr => "fr",
            Self::De => "de",
            Self::Es => "es",
            Self::It => "it",
        }
    }

    pub fn supported_codes() -> [&'static str; SUPPORTED_LOCALE_COUNT] {
        Self::SUPPORTED.map(Self::code)
    }

    fn normalize_code(code: &str) -> String {
        code.trim().replace('_', "-").to_ascii_lowercase()
    }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChatTextKey {
    ComposerPlaceholder,
    SendButton,
    StopButton,
    AttachButton,
    NewChatButton,
    HistoryButton,
    SettingsButton,
    VendorSelector,
    ModelSelector,
    ModeSelector,
    ThinkingSelector,
    PermissionModeSelector,
    UserRole,
    AssistantRole,
    ToolRole,
    SystemRole,
    EndpointLabel,
    OutputHandoff,
    RemoveAttachmentButton,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextCatalog {
    locale: ChatLocale,
    overrides: TextCatalogOverride,
}

impl TextCatalog {
    pub fn english() -> Self {
        Self::for_locale(ChatLocale::En)
    }

    pub fn for_locale(locale: ChatLocale) -> Self {
        Self {
            locale,
            overrides: TextCatalogOverride::default(),
        }
    }

    pub fn for_locale_code(code: &str) -> Result<Self, TextCatalogError> {
        Ok(Self::for_locale(ChatLocale::from_code(code)?))
    }

    pub fn with_text(mut self, key: ChatTextKey, text: impl Into<String>) -> Self {
        self.overrides.set_text(key, text.into());
        self
    }

    pub fn with_override_json(mut self, json: &str) -> Result<Self, TextCatalogError> {
        self.apply_override_json(json)?;
        Ok(self)
    }

    pub fn apply_override_json(&mut self, json: &str) -> Result<(), TextCatalogError> {
        let overrides = TextCatalogOverride::from_json(json)?;
        self.overrides.merge(overrides);
        Ok(())
    }

    pub fn resolve(&self, key: ChatTextKey) -> String {
        if let Some(text) = self.overrides.resolve(key) {
            return text.to_string();
        }
        locales::LocaleTexts::for_locale(self.locale)
            .resolve(key)
            .to_string()
    }

    pub fn locale(&self) -> ChatLocale {
        self.locale
    }
}

#[cfg(test)]
mod tests;
