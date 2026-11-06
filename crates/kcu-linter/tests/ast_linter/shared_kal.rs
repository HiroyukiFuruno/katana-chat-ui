use katana_ast_lint::KatanaAstLint;
use katana_ast_lint::config::KalConfig;
use kcu_linter::AstLintRunner;
use std::path::Path;

fn workspace_linter(root: &Path) -> Result<KatanaAstLint, String> {
    let mut config = KalConfig::load_from_path(&root.join("kal.json"))?;
    if config.source_roots.is_empty() {
        config.source_roots.push(root.to_path_buf());
    }
    /* WHY: Cargo はテストをメンバーのディレクトリで実行するため、KAL 0.5.4 に相対入力を渡さない。 */
    for input in config
        .source_roots
        .iter_mut()
        .chain(config.rules.values_mut().flat_map(|rule| &mut rule.inputs))
    {
        if input.is_relative() {
            *input = root.join(&*input);
        }
    }
    Ok(KatanaAstLint::with_config(config))
}

#[test]
fn repository_ast_lint() -> Result<(), String> {
    workspace_linter(&AstLintRunner::workspace_root()?)?.assert_clean();
    Ok(())
}

#[test]
fn relative_workspace_inputs_detect_both_length_violations() -> Result<(), String> {
    let workspace = tempfile::tempdir().map_err(|error| error.to_string())?;
    let root = workspace.path();
    std::fs::create_dir(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(
        root.join("src/probe.rs"),
        "#[cfg(test)]\nmod tests;\nfn long() {\n\n}\nfn after_tests() {}\n",
    )
    .map_err(|error| error.to_string())?;
    std::fs::write(
        root.join("kal.json"),
        r#"{
            "source_roots":["src"],
            "rules":{
                "file-length":{"threshold":3,"inputs":["src"]},
                "function-length":{"threshold":1}
            }
        }"#,
    )
    .map_err(|error| error.to_string())?;
    let violations = workspace_linter(root)?.violations();
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().any(|v| v.message.contains("current: 4")));
    assert!(
        violations
            .iter()
            .any(|v| v.message.contains("Function `long`"))
    );
    Ok(())
}
