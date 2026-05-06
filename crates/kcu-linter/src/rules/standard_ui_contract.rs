use crate::Violation;
use crate::ast::{SpanLocator, TestFileMatcher};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

mod matcher;

use matcher::StandardUiContractMatcher;

const CHAT_SURFACE_NAME: &str = "ChatUiSurface";

pub struct StandardUiContractRule;

impl StandardUiContractRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        if TestFileMatcher::is_test_file(path) {
            return Vec::new();
        }
        let mut visitor = StandardUiContractVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct StandardUiContractVisitor {
    file: PathBuf,
    matcher: StandardUiContractMatcher,
    violations: Vec<Violation>,
}

impl StandardUiContractVisitor {
    fn new(file: PathBuf) -> Self {
        let matcher = StandardUiContractMatcher::new(&file);
        Self {
            file,
            matcher,
            violations: Vec::new(),
        }
    }

    fn check_function(&mut self, signature: &syn::Signature) {
        if !self.matcher.is_floem_renderer()
            || !StandardUiContractMatcher::returns_string(&signature.output)
        {
            return;
        }
        if !StandardUiContractMatcher::is_standard_ui_text_collapse(signature) {
            return;
        }
        self.push_violation(
            signature.ident.span(),
            "標準チャットUIを String に潰す描画関数は禁止です。message / vendor / composer を部品として描画してください。",
        );
    }

    fn check_chat_surface(&mut self, node: &syn::ItemStruct) {
        if node.ident != CHAT_SURFACE_NAME {
            return;
        }
        let missing_fields = StandardUiContractMatcher::missing_chat_surface_fields(node);
        if missing_fields.is_empty() {
            return;
        }
        self.push_violation(
            node.ident.span(),
            format!(
                "ChatUiSurface の標準UI契約が不足しています: {}",
                missing_fields.join(", ")
            ),
        );
    }

    fn check_manual_literal(&mut self, node: &syn::LitStr) {
        if !self.matcher.is_manual_host()
            || !StandardUiContractMatcher::claims_manual_completion(&node.value())
        {
            return;
        }
        self.push_violation(
            node.span(),
            "manual-host の文言だけで標準UI完了扱いする表現は禁止です。標準UI部品の検証に分離してください。",
        );
    }

    fn check_standard_widget_literal(&mut self, node: &syn::LitStr) {
        if !self.matcher.is_standard_widget_file()
            || !StandardUiContractMatcher::is_prohibited_standard_widget_literal(&node.value())
        {
            return;
        }
        self.push_violation(
            node.span(),
            "標準 chat UI に debug / output JSON 表示を直接混ぜないでください。",
        );
    }

    fn check_manual_ident(&mut self, ident: &syn::Ident) {
        if !self.matcher.is_manual_host()
            || !StandardUiContractMatcher::claims_manual_completion(&ident.to_string())
        {
            return;
        }
        self.push_violation(
            ident.span(),
            "manual-host の命名だけで標準UI完了扱いする表現は禁止です。",
        );
    }

    fn check_standard_widget_ident(&mut self, ident: &syn::Ident) {
        if !self.matcher.is_standard_widget_file()
            || !StandardUiContractMatcher::is_prohibited_standard_widget_identifier(
                &ident.to_string(),
            )
        {
            return;
        }
        self.push_violation(
            ident.span(),
            "標準 chat UI の toolbar に settings/debug button を戻さないでください。",
        );
    }

    fn check_tool_send_icon_override(&mut self, node: &syn::ExprMethodCall) {
        if !self.matcher.is_tool_source() || node.method != "override_icon" {
            return;
        }
        if !node
            .args
            .iter()
            .any(StandardUiContractMatcher::is_send_icon_override_arg)
        {
            return;
        }
        self.push_violation(
            node.span(),
            "検証用 host で標準 send icon を上書きしないでください。上書き機能は unit test に分離してください。",
        );
    }

    fn check_debug_popup_fixed_width(&mut self, node: &syn::ExprMethodCall) {
        if !self.matcher.is_manual_host() || node.method != "width" {
            return;
        }
        if !node
            .args
            .iter()
            .any(StandardUiContractMatcher::is_debug_popup_width_arg)
        {
            return;
        }
        self.push_violation(
            node.span(),
            "debug popup を固定 width にしないでください。標準 chat UI の resize 検証を邪魔します。",
        );
    }

    fn push_violation(&mut self, span: proc_macro2::Span, message: impl Into<String>) {
        let (line, column) = SpanLocator::locate(span);
        self.violations
            .push(Violation::new(self.file.clone(), line, column, message));
    }
}

impl<'ast> Visit<'ast> for StandardUiContractVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_function(&node.sig);
        self.check_manual_ident(&node.sig.ident);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_function(&node.sig);
        self.check_manual_ident(&node.sig.ident);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_chat_surface(node);
        self.check_manual_ident(&node.ident);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        self.check_manual_literal(node);
        self.check_standard_widget_literal(node);
        syn::visit::visit_lit_str(self, node);
    }

    fn visit_ident(&mut self, node: &'ast syn::Ident) {
        self.check_standard_widget_ident(node);
        syn::visit::visit_ident(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.check_tool_send_icon_override(node);
        self.check_debug_popup_fixed_width(node);
        syn::visit::visit_expr_method_call(self, node);
    }
}
