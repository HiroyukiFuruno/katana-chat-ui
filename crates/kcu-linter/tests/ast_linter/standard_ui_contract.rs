use crate::support::{Fixture, floem_widget_path, manual_host_path, manual_host_src_dirs};
use kcu_linter::AstLintRunner;
use kcu_linter::rules::StandardUiContractRule;

#[test]
fn ast_linter_standard_ui_contract() -> Result<(), String> {
    let root = Fixture::root()?;
    let mut target_dirs = AstLintRunner::target_src_dirs(&root);
    target_dirs.extend(manual_host_src_dirs(&root));
    AstLintRunner::run(
        "standard-ui-contract",
        "katana-chat-ui は標準チャットUI部品を提供し、manual-host や文字列化描画で代替しないでください。",
        &target_dirs,
        StandardUiContractRule::lint,
    );
    Ok(())
}

#[test]
fn detects_standard_ui_contract_examples() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn message_text(surface: &ChatUiSurface) -> String {
            String::new()
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(floem_widget_path(), &syntax).len(),
        1
    );

    let syntax = Fixture::parse(
        r#"
        pub struct ChatUiSurface {
            pub messages: Vec<ChatUiMessageSurface>,
            pub composer: ChatUiComposerSurface,
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(Fixture::path(), &syntax).len(),
        1
    );
    Ok(())
}

#[test]
fn accepts_chat_ui_surface_standard_contract() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        pub struct ChatUiSurface {
            pub messages: Vec<ChatUiMessageSurface>,
            pub composer: ChatUiComposerSurface,
            pub usage: ChatUiUsageSurface,
            pub vendor_controls: VendorControlRenderModel,
        }
        "#,
    )?;
    assert!(StandardUiContractRule::lint(Fixture::path(), &syntax).is_empty());
    Ok(())
}

#[test]
fn detects_manual_host_standard_ui_leaks() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        struct StandardUiCompleteHost;

        fn label() -> &'static str {
            "katana-chat-ui standard UI complete"
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(manual_host_path(), &syntax).len(),
        2
    );

    let syntax = Fixture::parse(
        r#"
        fn popup() {
            style.width(DEBUG_POPUP_WIDTH);
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(manual_host_path(), &syntax).len(),
        1
    );
    Ok(())
}
