use kcu_linter::AstLintRunner;
use kcu_linter::rules::{
    AcpContractRule, CommentStyleRule, ErrorFirstRule, FileLengthRule, FunctionLengthRule,
    HttpStatusRule, MagicNumberRule, NestingDepthRule, ProcessCommandRule, ProhibitedAttributeRule,
    ProhibitedMacroRule, ProhibitedMethodRule, ProhibitedTypeRule, PublicFreeFunctionRule,
    StandardUiContractRule, TypeSeparationRule, VendorUiContractRule,
};
use std::path::{Path, PathBuf};

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

fn floem_widget_path() -> &'static Path {
    Path::new("crates/katana-chat-ui-floem/src/widget.rs")
}

fn manual_host_path() -> &'static Path {
    Path::new("tools/manual-host-floem/src/main.rs")
}

fn manual_host_src_dirs(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("tools/manual-host-egui/src"),
        root.join("tools/manual-host-floem/src"),
        root.join("tools/manual-host-gpui/src"),
    ]
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
fn ast_linter_type_separation() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "type-separation",
        "公開型と実装ロジックが大きいファイルに同居する場合は、責務単位で分離してください。",
        &AstLintRunner::target_src_dirs(&root),
        TypeSeparationRule::lint,
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
fn ast_linter_error_first() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "error-first",
        "`if let Ok(...)` で成功パスを包まず、`?` または早期 return を使ってください。",
        &AstLintRunner::target_src_dirs(&root),
        ErrorFirstRule::lint,
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
fn ast_linter_comment_style() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "comment-style",
        "通常の `//` コメントではなく、理由が分かる `/* WHY: ... */` を使ってください。",
        &AstLintRunner::target_src_dirs(&root),
        CommentStyleRule::lint,
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
fn ast_linter_magic_numbers() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "magic-number",
        "数値リテラルは名前付き定数へ出してください。",
        &AstLintRunner::target_src_dirs(&root),
        MagicNumberRule::lint,
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
fn ast_linter_prohibited_types() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "prohibited-types",
        "型消去や非構造化 JSON を避け、専用型へ分離してください。",
        &AstLintRunner::target_src_dirs(&root),
        ProhibitedTypeRule::lint,
    );
    Ok(())
}

#[test]
fn ast_linter_process_command_boundary() -> Result<(), String> {
    let root = Fixture::root()?;
    AstLintRunner::run(
        "process-command",
        "`std::process::Command::new` を直接使わず、専用の境界型へ集約してください。",
        &AstLintRunner::target_src_dirs(&root),
        ProcessCommandRule::lint,
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

    let syntax = Fixture::parse("fn helper() { if let Ok(value) = load() { value; } }")?;
    assert_eq!(ErrorFirstRule::lint(Fixture::path(), &syntax).len(), 1);

    let syntax = Fixture::parse("fn helper() { let value = 42; }")?;
    assert_eq!(MagicNumberRule::lint(Fixture::path(), &syntax).len(), 1);

    let syntax =
        Fixture::parse("fn helper() { let command = std::process::Command::new(\"ls\"); }")?;
    assert_eq!(ProcessCommandRule::lint(Fixture::path(), &syntax).len(), 1);

    let syntax = Fixture::parse("struct Holder { value: Box<dyn std::any::Any> }")?;
    assert_eq!(ProhibitedTypeRule::lint(Fixture::path(), &syntax).len(), 1);
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

#[test]
fn detects_floem_standard_ui_text_collapse() -> Result<(), syn::Error> {
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
    Ok(())
}

#[test]
fn detects_floem_output_handoff_text_collapse() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        pub fn output_handoff_text(output: &ChatUiOutputHandoffSurface) -> String {
            output.label.clone()
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(floem_widget_path(), &syntax).len(),
        1
    );
    Ok(())
}

#[test]
fn detects_thin_chat_ui_surface_contract() -> Result<(), syn::Error> {
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
            pub settings_icon: SvgIcon,
        }
        "#,
    )?;
    assert!(StandardUiContractRule::lint(Fixture::path(), &syntax).is_empty());
    Ok(())
}

#[test]
fn detects_manual_host_completion_claim() -> Result<(), syn::Error> {
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
    Ok(())
}

#[test]
fn detects_standard_widget_toolbar_settings_button() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn toolbar_settings_button() {}
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(floem_widget_path(), &syntax).len(),
        1
    );
    Ok(())
}

#[test]
fn detects_standard_widget_debug_or_settings_surface() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn debug_popup() -> &'static str {
            "output JSON"
        }

        fn header(surface: ChatUiSurface) {
            let _icon = surface.settings_icon;
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(floem_widget_path(), &syntax).len(),
        3
    );
    Ok(())
}

#[test]
fn detects_tool_send_icon_override() -> Result<(), syn::Error> {
    let syntax = Fixture::parse(
        r#"
        fn configure(session: &mut ChatSession) {
            session.override_icon(SvgIcon::new("send", SEND_OVERRIDE_ICON));
        }
        "#,
    )?;
    assert_eq!(
        StandardUiContractRule::lint(manual_host_path(), &syntax).len(),
        1
    );
    Ok(())
}

#[test]
fn detects_manual_debug_popup_fixed_width() -> Result<(), syn::Error> {
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

#[test]
fn ast_linter_v010_plan_blocks_api_only_completion() -> Result<(), String> {
    let root = Fixture::root()?;
    let proposal = read_workspace_file(
        &root,
        "openspec/changes/v0-1-0-chat-widget-floem/proposal.md",
    )?;
    let design = read_workspace_file(&root, "openspec/changes/v0-1-0-chat-widget-floem/design.md")?;
    let spec = read_workspace_file(
        &root,
        "openspec/changes/v0-1-0-chat-widget-floem/specs/chat-ui-foundation/spec.md",
    )?;
    let tasks = read_workspace_file(&root, "openspec/changes/v0-1-0-chat-widget-floem/tasks.md")?;

    assert_contains(
        &proposal,
        "API だけを使って独自 UI を作る利用方法は許容する",
    );
    assert_contains(&design, "descriptor / API だけでは完了としない");
    assert_contains(&spec, "API-only の利用は");
    assert_contains(&tasks, "API-only 実装だけを v0.1.0 完了扱いしない");
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

fn read_workspace_file(root: &Path, path: &str) -> Result<String, String> {
    let absolute_path = root.join(path);
    std::fs::read_to_string(&absolute_path)
        .map_err(|it| format!("{} を読めません: {it}", absolute_path.display()))
}

fn assert_contains(content: &str, expected: &str) {
    assert!(
        content.contains(expected),
        "`{expected}` が見つかりません。API-only 完了扱いを防ぐ contract を維持してください。"
    );
}
