use super::{ChatLocale, ChatTextKey, ChatTextSet, TextCatalog};

#[test]
fn english_catalog_provides_default_ui_text() {
    let texts = ChatTextSet::from_catalog(&TextCatalog::english());

    assert_eq!(texts.locale, "en");
    assert_eq!(texts.send_button, "Send");
    assert_eq!(texts.composer_placeholder, "Ask anything");
}

#[test]
fn catalog_allows_later_locale_override_without_widget_branching() {
    let catalog = TextCatalog::english()
        .with_text(ChatTextKey::ComposerPlaceholder, "Message Claude")
        .with_text(ChatTextKey::ThinkingSelector, "Effort");

    let texts = ChatTextSet::from_catalog(&catalog);

    assert_eq!(texts.composer_placeholder, "Message Claude");
    assert_eq!(texts.thinking_selector, "Effort");
}

#[test]
fn supported_locale_codes_match_katana_languages() {
    assert_eq!(
        ChatLocale::supported_codes(),
        [
            "en", "ja", "zh-CN", "zh-TW", "ko", "pt", "fr", "de", "es", "it",
        ]
    );
}

#[test]
fn locale_code_selects_builtin_texts() -> Result<(), super::TextCatalogError> {
    let texts = ChatTextSet::from_catalog(&TextCatalog::for_locale_code("ja")?);

    assert_eq!(texts.locale, "ja");
    assert_eq!(texts.send_button, "送信");
    assert_eq!(texts.settings_button, "設定");
    Ok(())
}

#[test]
fn locale_code_normalizes_common_separator_variants() -> Result<(), super::TextCatalogError> {
    let texts = ChatTextSet::from_catalog(&TextCatalog::for_locale_code("zh_CN")?);

    assert_eq!(texts.locale, "zh-CN");
    assert_eq!(texts.send_button, "发送");
    Ok(())
}

#[test]
fn override_json_changes_only_requested_texts() -> Result<(), super::TextCatalogError> {
    let catalog = TextCatalog::for_locale(ChatLocale::Ja)
        .with_override_json(r#"{"send_button":"Run","composer_placeholder":"Ask local model"}"#)?;
    let texts = ChatTextSet::from_catalog(&catalog);

    assert_eq!(texts.locale, "ja");
    assert_eq!(texts.send_button, "Run");
    assert_eq!(texts.composer_placeholder, "Ask local model");
    assert_eq!(texts.stop_button, "停止");
    Ok(())
}

#[test]
fn override_json_rejects_unknown_text_keys() {
    let result = TextCatalog::english().with_override_json(r#"{"unknown_label":"Run"}"#);

    assert!(matches!(
        result,
        Err(super::TextCatalogError::InvalidOverrideJson(_))
    ));
}
