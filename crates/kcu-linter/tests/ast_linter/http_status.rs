use crate::support::Fixture;
use kcu_linter::AstLintRunner;
use kcu_linter::rules::HttpStatusRule;

#[test]
fn ast_linter_http_status_before_json() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "http-status-before-json",
        "HTTP 応答は `.json()` の前に `.error_for_status()` を通してください。",
        &AstLintRunner::target_src_dirs(&root),
        HttpStatusRule::lint,
    );
    Ok(())
}

#[test]
fn detects_http_json_without_status_check() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        async fn load(client: reqwest::Client) -> Result<(), reqwest::Error> {
            let response = client.post("http://example.com").send().await?;
            let _: serde_json::Value = response.json().await?;
            Ok(())
        }
        "#,
    )?;
    assert_eq!(HttpStatusRule::lint(Fixture::path(), &syntax).len(), 1);
    Ok(())
}
