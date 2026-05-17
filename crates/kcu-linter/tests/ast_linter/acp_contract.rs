use crate::support::Fixture;
use kcu_linter::AstLintRunner;
use kcu_linter::rules::AcpContractRule;

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
