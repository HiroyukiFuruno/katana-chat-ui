#[path = "ast_linter/acp_contract.rs"]
mod acp_contract;
#[path = "ast_linter/http_status.rs"]
mod http_status;
#[path = "ast_linter/plan_guards.rs"]
mod plan_guards;
#[path = "ast_linter/prohibited_method.rs"]
mod prohibited_method;
#[path = "ast_linter/standard_ui_contract.rs"]
mod standard_ui_contract;
#[path = "ast_linter/support.rs"]
mod support;
#[path = "ast_linter/vendor_ui_contract.rs"]
mod vendor_ui_contract;

#[test]
fn repository_ast_lint() {
    katana_ast_lint::KatanaAstLint::from_workspace().assert_clean();
}
