use crate::{ChatLocale, ChatSession, TextCatalog, TextCatalogError};

impl ChatSession {
    pub fn set_locale(&mut self, locale: ChatLocale) {
        self.texts = TextCatalog::for_locale(locale);
    }

    pub fn set_locale_code(&mut self, code: &str) -> Result<(), TextCatalogError> {
        self.set_locale(ChatLocale::from_code(code)?);
        Ok(())
    }

    pub fn apply_text_override_json(&mut self, json: &str) -> Result<(), TextCatalogError> {
        self.texts.apply_override_json(json)
    }
}
