use crate::render_model::ChatUiOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ChatUiConfig {
    #[serde(default)]
    pub options: ChatUiOptions,
}

impl ChatUiConfig {
    pub fn new(options: ChatUiOptions) -> Self {
        Self { options }
    }

    pub fn with_options(mut self, options: ChatUiOptions) -> Self {
        self.options = options;
        self
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
        let config: ChatUiConfig = serde_json::from_str(r#"{"options":{"debug":true}}"#)?;

        assert!(config.options.debug);
        Ok(())
    }

    #[test]
    fn json_options_default_missing_debug_to_false() -> Result<(), serde_json::Error> {
        let config: ChatUiConfig = serde_json::from_str(r#"{"options":{}}"#)?;

        assert_eq!(config.options, ChatUiOptions::default());
        Ok(())
    }

    #[test]
    fn json_options_reject_null_options() {
        let result = serde_json::from_str::<ChatUiConfig>(r#"{"options":null}"#);

        assert!(result.is_err());
    }
}
