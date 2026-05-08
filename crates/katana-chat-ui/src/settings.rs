use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChatSettingsError {
    #[error("settings path must not be empty")]
    EmptyPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSettingsReference {
    pub path: String,
}

impl ChatSettingsReference {
    pub fn new(path: impl Into<String>) -> Result<Self, ChatSettingsError> {
        let path = path.into();
        if path.trim().is_empty() {
            return Err(ChatSettingsError::EmptyPath);
        }
        Ok(Self { path })
    }

    pub fn default_path() -> Self {
        Self {
            path: ".katana-chat-ui/settings.json".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSettingsPatch {
    pub theme: String,
    pub locale: String,
    pub placeholder: String,
    pub svg_icon_overrides: Vec<SvgIconOverride>,
    pub provider_display_order: Vec<String>,
    pub composer_behavior: String,
}

impl ChatSettingsPatch {
    pub fn empty() -> Self {
        Self {
            theme: String::new(),
            locale: String::new(),
            placeholder: String::new(),
            svg_icon_overrides: Vec::new(),
            provider_display_order: Vec::new(),
            composer_behavior: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvgIconOverride {
    pub icon_id: String,
    pub svg: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSettingsMergeIntent {
    pub target_path: String,
    pub patch: ChatSettingsPatch,
}

impl ChatSettingsMergeIntent {
    pub fn new(reference: &ChatSettingsReference, patch: ChatSettingsPatch) -> Self {
        Self {
            target_path: reference.path.clone(),
            patch,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSettingsRenderModel {
    pub visible: bool,
    pub reference: ChatSettingsReference,
    pub sections: Vec<ChatSettingsSection>,
}

impl ChatSettingsRenderModel {
    pub fn closed(reference: ChatSettingsReference) -> Self {
        Self {
            visible: false,
            reference,
            sections: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSettingsSection {
    pub id: String,
    pub label: String,
    pub items: Vec<ChatSettingsItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatSettingsItem {
    pub id: String,
    pub label: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::{
        ChatSettingsMergeIntent, ChatSettingsPatch, ChatSettingsReference, SvgIconOverride,
    };

    #[test]
    fn settings_reference_rejects_empty_path() {
        assert!(ChatSettingsReference::new("  ").is_err());
    }

    #[test]
    fn merge_intent_targets_required_settings_file() -> Result<(), super::ChatSettingsError> {
        let reference = ChatSettingsReference::new("/tmp/kcu-settings.json")?;
        let mut patch = ChatSettingsPatch::empty();
        patch.theme = "dark".to_string();
        patch.svg_icon_overrides.push(SvgIconOverride {
            icon_id: "send".to_string(),
            svg: "<svg/>".to_string(),
        });

        let intent = ChatSettingsMergeIntent::new(&reference, patch);

        assert_eq!(intent.target_path, "/tmp/kcu-settings.json");
        assert_eq!(intent.patch.svg_icon_overrides[0].icon_id, "send");
        Ok(())
    }
}
