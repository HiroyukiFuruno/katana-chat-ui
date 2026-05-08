use super::{ChatLocale, ChatTextKey, locale_values};

pub(super) const TEXT_KEY_COUNT: usize = 19;

#[derive(Debug, Clone, Copy)]
pub(super) struct LocaleTexts {
    pub(super) values: [&'static str; TEXT_KEY_COUNT],
}

impl LocaleTexts {
    pub(super) fn for_locale(locale: ChatLocale) -> Self {
        match locale {
            ChatLocale::En => locale_values::EN,
            ChatLocale::Ja => locale_values::JA,
            ChatLocale::ZhCn => locale_values::ZH_CN,
            ChatLocale::ZhTw => locale_values::ZH_TW,
            ChatLocale::Ko => locale_values::KO,
            ChatLocale::Pt => locale_values::PT,
            ChatLocale::Fr => locale_values::FR,
            ChatLocale::De => locale_values::DE,
            ChatLocale::Es => locale_values::ES,
            ChatLocale::It => locale_values::IT,
        }
    }

    pub(super) fn resolve(self, key: ChatTextKey) -> &'static str {
        self.values[text_index(key)]
    }
}

fn text_index(key: ChatTextKey) -> usize {
    key as usize
}
