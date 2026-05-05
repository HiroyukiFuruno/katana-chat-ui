use kcu_linter::AstLintRunner;
use kcu_linter::rules::{
    AcpContractRule, FileLengthRule, FunctionLengthRule, HttpStatusRule, NestingDepthRule,
    ProhibitedAttributeRule, ProhibitedMacroRule, ProhibitedMethodRule, PublicFreeFunctionRule,
};
use std::path::Path;

struct Fixture;

impl Fixture {
    fn parse(code: &str) -> Result<syn::File, syn::Error> {
        syn::parse_file(code)
    }

    fn path() -> &'static Path {
        Path::new("fixture.rs")
    }

    fn root() -> Result<std::path::PathBuf, String> {
        AstLintRunner::workspace_root()
    }
}

#[test]
fn ast_linter_file_length() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "file-length",
        "200 行を超えるファイルは責務単位で分割してください。",
        &AstLintRunner::target_src_dirs(&root),
        FileLengthRule::lint,
    );
    Ok(())
}

#[test]
fn ast_linter_function_length() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "function-length",
        "30 行を超える関数は helper method に分割してください。",
        &AstLintRunner::target_src_dirs(&root),
        FunctionLengthRule::lint,
    );
    Ok(())
}

#[test]
fn ast_linter_nesting_depth() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "nesting-depth",
        "ネストは 3 段以内に抑え、早期 return か helper method に分けてください。",
        &AstLintRunner::target_src_dirs(&root),
        NestingDepthRule::lint,
    );
    Ok(())
}

#[test]
fn ast_linter_public_free_function() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "public-free-function",
        "公開 API は struct と impl に集約してください。",
        &AstLintRunner::target_src_dirs(&root),
        PublicFreeFunctionRule::lint,
    );
    Ok(())
}

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
fn ast_linter_prohibited_macros() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "prohibited-macros",
        "`todo!` / `unimplemented!` / `dbg!` を残さないでください。",
        &AstLintRunner::target_src_dirs(&root),
        ProhibitedMacroRule::lint,
    );
    Ok(())
}

#[test]
fn ast_linter_prohibited_attributes() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "prohibited-attributes",
        "`#[allow(...)]` で検査を緩めず、原因を修正してください。",
        &AstLintRunner::target_src_dirs(&root),
        ProhibitedAttributeRule::lint,
    );
    Ok(())
}

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
fn ast_linter_ai_provider_uses_request_context() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "ai-provider-request-context",
        "`AiProvider::execute` は `AiRequest.context` をプロンプト構築へ反映してください。",
        &AstLintRunner::target_src_dirs(&root),
        AcpContractRule::lint,
    );
    Ok(())
}

#[test]
fn detects_rule_examples() -> Result<(), syn::Error> {
    let syntax = Fixture::parse("pub fn helper() { todo!() }")?;
    assert_eq!(
        PublicFreeFunctionRule::lint(Fixture::path(), &syntax).len(),
        1
    );
    assert_eq!(ProhibitedMacroRule::lint(Fixture::path(), &syntax).len(), 1);

    let syntax = Fixture::parse("fn helper(value: Option<String>) { value.unwrap_or_default(); }")?;
    assert_eq!(
        ProhibitedMethodRule::lint(Fixture::path(), &syntax).len(),
        1
    );

    let syntax = Fixture::parse("#[allow(dead_code)] struct Sample;")?;
    assert_eq!(
        ProhibitedAttributeRule::lint(Fixture::path(), &syntax).len(),
        1
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

#[test]
fn accepts_ai_provider_context_through_helper_chain() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        impl AiProvider for DemoProvider {
            async fn execute(&self, request: &AiRequest) -> Result<AiResponse, AcpError> {
                let message = Self::build_messages(request);
                Ok(AiResponse { content: message })
            }

            fn build_messages(request: &AiRequest) -> String {
                Self::build_system_message(request)
            }

            fn build_system_message(request: &AiRequest) -> String {
                Self::build_context_message(request)
            }

            fn build_context_message(request: &AiRequest) -> String {
                format!("{} {}", request.context.uri, request.context.content)
            }
        }
        "#,
    )?;
    assert!(AcpContractRule::lint(Fixture::path(), &syntax).is_empty());
    Ok(())
}

#[test]
fn detects_deep_nesting_example() -> Result<(), syn::Error> {
    let syntax = Fixture::parse("fn demo() { if true { if true { if true { if true {} } } } }")?;
    assert_eq!(NestingDepthRule::lint(Fixture::path(), &syntax).len(), 1);
    Ok(())
}
