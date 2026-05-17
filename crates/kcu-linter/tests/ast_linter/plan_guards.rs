use crate::support::{Fixture, assert_contains, read_workspace_file};

#[test]
fn ast_linter_v010_plan_blocks_api_only_completion() -> Result<(), String> {
    let root = Fixture::root()?;
    let scope = read_workspace_file(&root, "openspec/v0-1-0-scope.md")?;

    assert_contains(
        &scope,
        "利用側が独自に AI エージェントチャット画面を作らなくても使える標準 UI",
    );
    assert_contains(&scope, "Markdown ファイルを生成できる");
    assert_contains(&scope, "Markdown ファイルを編集できる");
    assert_contains(&scope, "生成、編集した変更を元に戻せる");
    assert_contains(&scope, "Ollama は provider selector に出さない");
    Ok(())
}

#[test]
fn ast_linter_floem_standard_ui_crate_is_not_descriptor_only() -> Result<(), String> {
    let root = Fixture::root()?;
    let manifest = read_workspace_file(&root, "crates/katana-chat-ui-floem/Cargo.toml")?;
    let lib = read_workspace_file(&root, "crates/katana-chat-ui-floem/src/lib.rs")?;
    let widget = read_workspace_file(&root, "crates/katana-chat-ui-floem/src/widget.rs")?;

    assert_contains(&manifest, "floem =");
    assert_contains(&lib, "pub use widget::FloemChatView");
    assert_contains(&widget, "pub struct FloemChatView");
    assert_contains(&widget, "pub fn render");
    Ok(())
}
