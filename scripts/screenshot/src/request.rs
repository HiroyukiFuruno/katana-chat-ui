use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::Path;

const SUPPORTED_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub schema_version: String,
    pub name: String,
    #[serde(default)]
    pub viewport: Viewport,
    #[serde(default)]
    pub scenario: Scenario,
    #[serde(default = "default_output_name")]
    pub output_name: String,
    pub baseline_sha256: String,
}

impl Request {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            bail!("unsupported schema_version: {}", self.schema_version);
        }
        validate_sha256(&self.baseline_sha256)?;
        self.viewport.validate()?;
        self.scenario.validate()
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    fn validate(&self) -> Result<()> {
        if self.width < 320 || self.height < 240 {
            bail!("viewport must be at least 320x240");
        }
        Ok(())
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1440,
            height: 900,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    #[serde(default = "default_provider_id")]
    pub provider_id: String,
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_draft")]
    pub draft: String,
    #[serde(default)]
    pub kind: ScenarioKind,
    #[serde(default)]
    pub debug: bool,
}

impl Scenario {
    fn validate(&self) -> Result<()> {
        if !supported_provider_ids().contains(&self.provider_id.as_str()) {
            bail!("unsupported provider_id: {}", self.provider_id);
        }
        Ok(())
    }
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            provider_id: default_provider_id(),
            title: default_title(),
            draft: default_draft(),
            kind: ScenarioKind::default(),
            debug: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    Empty,
    #[default]
    Conversation,
    Thinking,
    OutputDebug,
}

pub fn load(path: &Path) -> Result<Request> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read request: {}", path.display()))?;
    let request: Request = serde_json::from_str(&text)
        .with_context(|| format!("invalid request JSON: {}", path.display()))?;
    request.validate()?;
    Ok(request)
}

pub fn supported_provider_ids() -> Vec<&'static str> {
    vec!["claude-code", "codex-cli", "github-copilot", "opencode"]
}

fn default_output_name() -> String {
    "katana-chat-ui.png".to_string()
}

fn default_provider_id() -> String {
    "claude-code".to_string()
}

fn default_title() -> String {
    "katana-chat-ui".to_string()
}

fn default_draft() -> String {
    "Ask anything".to_string()
}

fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64 || !value.chars().all(|it| it.is_ascii_hexdigit()) {
        bail!("baseline_sha256 must be a 64-character hex digest");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Request, ScenarioKind};

    #[test]
    fn parses_minimal_request_with_safe_defaults() -> Result<(), serde_json::Error> {
        let request: Request = serde_json::from_str(
            r#"{
                "schema_version": "1",
                "name": "minimal",
                "baseline_sha256": "06a6c7aa528a104e168a1539487e4ee70865fc6c6429319592f9776b82468848"
            }"#,
        )?;

        assert_eq!(request.scenario.provider_id, "claude-code");
        assert!(matches!(request.scenario.kind, ScenarioKind::Conversation));
        assert_eq!(request.output_name, "katana-chat-ui.png");
        Ok(())
    }

    #[test]
    fn parses_visual_matrix_scenario_kinds() -> Result<(), serde_json::Error> {
        for (kind, expected) in [
            ("empty", ScenarioKind::Empty),
            ("conversation", ScenarioKind::Conversation),
            ("thinking", ScenarioKind::Thinking),
            ("output_debug", ScenarioKind::OutputDebug),
        ] {
            let request: Request = serde_json::from_str(&format!(
                r#"{{
                    "schema_version": "1",
                    "name": "{kind}",
                    "scenario": {{ "kind": "{kind}" }},
                    "baseline_sha256": "06a6c7aa528a104e168a1539487e4ee70865fc6c6429319592f9776b82468848"
                }}"#
            ))?;
            assert_eq!(request.scenario.kind, expected);
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_baseline_hash() -> Result<(), serde_json::Error> {
        let request: Request = serde_json::from_str(
            r#"{
                "schema_version": "1",
                "name": "invalid",
                "baseline_sha256": "not-a-sha"
            }"#,
        )?;

        assert!(request.validate().is_err());
        Ok(())
    }
}
