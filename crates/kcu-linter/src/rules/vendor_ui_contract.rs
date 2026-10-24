use crate::Violation;
use crate::ast::{SpanLocator, TestFileMatcher};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

const VENDOR_ID_FIELDS: [&str; 2] = ["vendor_id", "active_vendor_id"];
const OFFICIAL_URL_FIELD: &str = "official_url";

pub struct VendorUiContractRule;

impl VendorUiContractRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        if TestFileMatcher::is_test_file(path) {
            return Vec::new();
        }
        let mut visitor = VendorUiContractVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct VendorUiContractVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl VendorUiContractVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_vendor_id_comparison(&mut self, node: &syn::ExprBinary) {
        if !Self::is_equality_operator(&node.op) {
            return;
        }
        if !Self::compares_vendor_id_to_literal(&node.left, &node.right) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.op.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "vendor_id の文字列直書き分岐は禁止です。公式根拠付き capability で表示可否を決めてください。",
        ));
    }

    fn check_capability_fact(&mut self, node: &syn::ExprStruct) {
        if !Self::is_capability_fact(node) || !Self::is_supported(node) {
            return;
        }
        if Self::has_official_url(node) {
            return;
        }
        let (line, column) = SpanLocator::locate(
            node.path
                .segments
                .last()
                .map_or(node.path.span(), |it| it.ident.span()),
        );
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "Supported capability には公式URLが必須です。official_url に根拠URLを入れてください。",
        ));
    }

    fn is_equality_operator(operator: &syn::BinOp) -> bool {
        matches!(operator, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
    }

    fn compares_vendor_id_to_literal(left: &syn::Expr, right: &syn::Expr) -> bool {
        (Self::is_vendor_id_expr(left) && Self::is_string_literal(right))
            || (Self::is_vendor_id_expr(right) && Self::is_string_literal(left))
    }

    fn is_vendor_id_expr(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Field(field) => Self::is_vendor_id_member(&field.member),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|it| Self::is_vendor_id_name(&it.to_string())),
            syn::Expr::Reference(reference) => Self::is_vendor_id_expr(&reference.expr),
            syn::Expr::Paren(paren) => Self::is_vendor_id_expr(&paren.expr),
            _ => false,
        }
    }

    fn is_vendor_id_member(member: &syn::Member) -> bool {
        match member {
            syn::Member::Named(ident) => Self::is_vendor_id_name(&ident.to_string()),
            syn::Member::Unnamed(_) => false,
        }
    }

    fn is_vendor_id_name(name: &str) -> bool {
        VENDOR_ID_FIELDS.contains(&name)
    }

    fn is_string_literal(expr: &syn::Expr) -> bool {
        matches!(
            expr,
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(_),
                ..
            })
        )
    }

    fn is_capability_fact(node: &syn::ExprStruct) -> bool {
        node.path
            .segments
            .last()
            .is_some_and(|it| it.ident.to_string().ends_with("CapabilityFact"))
    }

    fn is_supported(node: &syn::ExprStruct) -> bool {
        node.fields
            .iter()
            .filter_map(Self::named_field)
            .any(|(_, expr)| Self::expr_mentions_supported(expr))
    }

    fn named_field(field: &syn::FieldValue) -> Option<(String, &syn::Expr)> {
        match &field.member {
            syn::Member::Named(ident) => Some((ident.to_string(), &field.expr)),
            syn::Member::Unnamed(_) => None,
        }
    }

    fn expr_mentions_supported(expr: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = expr else {
            return false;
        };
        path.path
            .segments
            .last()
            .is_some_and(|it| it.ident == "Supported")
    }

    fn has_official_url(node: &syn::ExprStruct) -> bool {
        node.fields
            .iter()
            .filter_map(Self::named_field)
            .any(|(name, expr)| name == OFFICIAL_URL_FIELD && Self::expr_has_url(expr))
    }

    fn expr_has_url(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Call(call) => Self::call_has_url(call),
            syn::Expr::Lit(lit) => Self::literal_has_url(lit),
            syn::Expr::Path(path) => !path.path.is_ident("None"),
            _ => true,
        }
    }

    fn call_has_url(call: &syn::ExprCall) -> bool {
        if !Self::is_some_call(&call.func) {
            return true;
        }
        call.args.iter().any(Self::expr_has_url)
    }

    fn is_some_call(expr: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = expr else {
            return false;
        };
        path.path
            .segments
            .last()
            .is_some_and(|it| it.ident == "Some")
    }

    fn literal_has_url(lit: &syn::ExprLit) -> bool {
        match &lit.lit {
            syn::Lit::Str(value) => !value.value().trim().is_empty(),
            _ => true,
        }
    }
}

impl<'ast> Visit<'ast> for VendorUiContractVisitor {
    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        self.check_vendor_id_comparison(node);
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        self.check_capability_fact(node);
        syn::visit::visit_expr_struct(self, node);
    }
}
