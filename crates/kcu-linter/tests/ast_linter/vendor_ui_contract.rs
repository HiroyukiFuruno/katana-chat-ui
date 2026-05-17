use crate::support::Fixture;
use kcu_linter::AstLintRunner;
use kcu_linter::rules::VendorUiContractRule;

#[test]
fn ast_linter_vendor_ui_contract() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "vendor-ui-contract",
        "vendor UI の表示可否は vendor_id 直書きではなく、公式URL付き capability fact で決めてください。",
        &AstLintRunner::target_src_dirs(&root),
        VendorUiContractRule::lint,
    );
    Ok(())
}

#[test]
fn detects_vendor_id_literal_ui_branch() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn render(profile: VendorUiProfile) -> bool {
            profile.vendor_id == "claude"
        }
        "#,
    )?;
    assert_eq!(
        VendorUiContractRule::lint(Fixture::path(), &syntax).len(),
        1
    );
    Ok(())
}

#[test]
fn detects_supported_capability_fact_without_official_url() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn fact() -> VendorCapabilityFact {
            VendorCapabilityFact {
                capability: VendorCapability::WebSearch,
                support: VendorCapabilitySupport::Supported,
                official_url: None,
            }
        }
        "#,
    )?;
    assert_eq!(
        VendorUiContractRule::lint(Fixture::path(), &syntax).len(),
        1
    );
    Ok(())
}

#[test]
fn accepts_supported_capability_fact_with_official_url() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn fact() -> VendorCapabilityFact {
            VendorCapabilityFact {
                capability: VendorCapability::WebSearch,
                support: VendorCapabilitySupport::Supported,
                official_url: Some("https://docs.example.com/web-search"),
            }
        }
        "#,
    )?;
    assert!(VendorUiContractRule::lint(Fixture::path(), &syntax).is_empty());
    Ok(())
}
