use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use crate::{AcpProviderProfile, ChatHostError};

/// ACP vendor ID と host root から stdio 起動 profile を作る。
pub fn acp_provider_profile(
    vendor_id: &str,
    root: impl Into<PathBuf>,
) -> Result<AcpProviderProfile, ChatHostError> {
    let spec = launch_spec(vendor_id)?;
    let command = command_on_path(spec.command).ok_or_else(|| {
        ChatHostError::Provider(format!(
            "ACP command `{}` is missing or not executable on PATH for vendor `{vendor_id}`",
            spec.command
        ))
    })?;
    let profile = AcpProviderProfile::new(vendor_id, command, root).args(spec.args.iter().copied());
    if vendor_id == "claude-code" {
        Ok(profile.envs(claude_ollama_environment_from_process()))
    } else {
        Ok(profile)
    }
}

struct LaunchSpec {
    command: &'static str,
    args: &'static [&'static str],
}

fn launch_spec(vendor_id: &str) -> Result<LaunchSpec, ChatHostError> {
    match vendor_id {
        "codex-cli" => Ok(LaunchSpec {
            command: "npx",
            args: &["-y", "@agentclientprotocol/codex-acp@2.2.2"],
        }),
        "claude-code" => Ok(LaunchSpec {
            command: "npx",
            args: &["-y", "@agentclientprotocol/claude-agent-acp@0.95.0"],
        }),
        "github-copilot" => Ok(LaunchSpec {
            command: "copilot",
            args: &["--acp", "--stdio"],
        }),
        "katanagent" => Err(ChatHostError::Provider(
            "KatanAgent uses the direct Ollama provider and has no ACP profile".into(),
        )),
        _ => Err(ChatHostError::Provider(format!(
            "Unknown ACP provider `{vendor_id}`"
        ))),
    }
}

fn command_on_path(command: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths)
        .map(|directory| directory.join(command))
        .find(|candidate| is_executable(candidate))
}

fn claude_ollama_environment_from_process() -> BTreeMap<String, String> {
    let model = env::var("KCU_CLAUDE_OLLAMA_MODEL").ok();
    let endpoint = env::var("KCU_OLLAMA_ENDPOINT").ok();
    claude_ollama_environment(model.as_deref(), endpoint.as_deref())
}

fn claude_ollama_environment(
    configured_model: Option<&str>,
    configured_endpoint: Option<&str>,
) -> BTreeMap<String, String> {
    let Some(model) = configured_model
        .map(str::trim)
        .filter(|model| !model.is_empty())
    else {
        return BTreeMap::new();
    };
    let endpoint = configured_endpoint
        .map(str::trim)
        .filter(|endpoint| !endpoint.is_empty())
        .unwrap_or("http://localhost:11434");
    BTreeMap::from([
        ("ANTHROPIC_AUTH_TOKEN".into(), "ollama".into()),
        ("ANTHROPIC_API_KEY".into(), String::new()),
        ("ANTHROPIC_BASE_URL".into(), endpoint.into()),
        ("ANTHROPIC_MODEL".into(), model.into()),
        ("ANTHROPIC_CUSTOM_MODEL_OPTION".into(), model.into()),
    ])
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::{claude_ollama_environment, launch_spec};

    #[test]
    fn known_acp_vendors_use_the_documented_stdio_commands() -> Result<(), String> {
        let codex = launch_spec("codex-cli").map_err(|error| error.to_string())?;
        let claude = launch_spec("claude-code").map_err(|error| error.to_string())?;
        let copilot = launch_spec("github-copilot").map_err(|error| error.to_string())?;

        assert_eq!(codex.command, "npx");
        assert_eq!(codex.args, ["-y", "@agentclientprotocol/codex-acp@2.2.2"]);
        assert_eq!(claude.command, "npx");
        assert_eq!(
            claude.args,
            ["-y", "@agentclientprotocol/claude-agent-acp@0.95.0"]
        );
        assert_eq!(copilot.command, "copilot");
        assert_eq!(copilot.args, ["--acp", "--stdio"]);
        Ok(())
    }

    #[test]
    fn direct_ollama_and_unknown_vendors_do_not_receive_acp_profiles() {
        assert!(launch_spec("katanagent").is_err());
        assert!(launch_spec("unknown").is_err());
    }

    #[test]
    fn claude_ollama_profile_is_opt_in_and_scoped_to_the_configured_model() {
        assert!(claude_ollama_environment(None, None).is_empty());

        let environment =
            claude_ollama_environment(Some(" qwen3:4b "), Some(" http://127.0.0.1:11434 "));
        assert_eq!(
            environment.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&"ollama".to_string())
        );
        assert_eq!(environment.get("ANTHROPIC_API_KEY"), Some(&String::new()));
        assert_eq!(
            environment.get("ANTHROPIC_BASE_URL"),
            Some(&"http://127.0.0.1:11434".to_string())
        );
        assert_eq!(
            environment.get("ANTHROPIC_MODEL"),
            Some(&"qwen3:4b".to_string())
        );
        assert_eq!(
            environment.get("ANTHROPIC_CUSTOM_MODEL_OPTION"),
            Some(&"qwen3:4b".to_string())
        );
    }
}
