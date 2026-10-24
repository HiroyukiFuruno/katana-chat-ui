use crate::{ChatSettingsError, ChatSettingsReference, render_model::ChatUiOptions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUiConfig {
    pub settings_path: String,
    #[serde(default)]
    pub options: ChatUiOptions,
}

impl ChatUiConfig {
    pub fn new(
        settings_path: impl Into<String>,
        options: ChatUiOptions,
    ) -> Result<Self, ChatSettingsError> {
        let reference = ChatSettingsReference::new(settings_path)?;
        Ok(Self {
            settings_path: reference.path,
            options,
        })
    }

    pub fn with_options(mut self, options: ChatUiOptions) -> Self {
        self.options = options;
        self
    }

    pub fn settings_reference(&self) -> Result<ChatSettingsReference, ChatSettingsError> {
        ChatSettingsReference::new(self.settings_path.clone())
    }
}

impl Default for ChatUiConfig {
    fn default() -> Self {
        Self {
            settings_path: ChatSettingsReference::default_path().path,
            options: ChatUiOptions::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChatUiConfig;
    use crate::ChatUiOptions;

    #[test]
    fn default_options_disable_debug() {
        let config = ChatUiConfig::default();

        assert!(!config.options.debug);
    }

    #[test]
    fn json_options_accept_debug_flag() -> Result<(), serde_json::Error> {
        let config: ChatUiConfig = serde_json::from_str(
            r#"{"settings_path":"/tmp/kcu-settings.json","options":{"debug":true}}"#,
        )?;

        assert!(config.options.debug);
        Ok(())
    }

    #[test]
    fn json_options_default_missing_debug_to_false() -> Result<(), serde_json::Error> {
        let config: ChatUiConfig =
            serde_json::from_str(r#"{"settings_path":"/tmp/kcu-settings.json","options":{}}"#)?;

        assert_eq!(config.options, ChatUiOptions::default());
        Ok(())
    }

    #[test]
    fn json_config_rejects_missing_settings_path() {
        let result = serde_json::from_str::<ChatUiConfig>(r#"{"options":{}}"#);

        assert!(result.is_err());
    }

    #[test]
    fn settings_reference_rejects_empty_path() {
        let config = ChatUiConfig {
            settings_path: String::new(),
            options: ChatUiOptions::default(),
        };

        assert!(config.settings_reference().is_err());
    }

    #[test]
    fn json_options_reject_null_options() {
        let result = serde_json::from_str::<ChatUiConfig>(r#"{"options":null}"#);

        assert!(result.is_err());
    }
}
