use crate::support::Fixture;
use kcu_linter::AstLintRunner;
use kcu_linter::rules::ProhibitedMethodRule;

#[test]
fn ast_linter_prohibited_methods() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "prohibited-methods",
        "`unwrap` / `expect` / `unwrap_or_default` を使わず、失敗を明示的に扱ってください。",
        &AstLintRunner::target_src_dirs(&root),
        ProhibitedMethodRule::lint,
    );
    Ok(())
}

#[test]
fn detects_prohibited_method_example() -> Result<(), syn::Error> {
    let syntax = Fixture::parse("fn helper(value: Option<String>) { value.unwrap_or_default(); }")?;
    assert_eq!(
        ProhibitedMethodRule::lint(Fixture::path(), &syntax).len(),
        1
    );
    Ok(())
}
