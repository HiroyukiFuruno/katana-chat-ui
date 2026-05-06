use crate::Violation;
use crate::ast::{NumericLiteralMatcher, SpanLocator, TestFileMatcher};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct MagicNumberRule;

impl MagicNumberRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        if TestFileMatcher::is_test_file(path) {
            return Vec::new();
        }
        let mut visitor = MagicNumberVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct MagicNumberVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    const_depth: usize,
}

impl MagicNumberVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            const_depth: 0,
        }
    }

    fn check_literal(&mut self, literal: &syn::Lit) {
        if self.const_depth > 0 {
            return;
        }
        if let Some((value, span)) = Self::numeric_value(literal) {
            self.check_value(value, span);
        }
    }

    fn numeric_value(literal: &syn::Lit) -> Option<(f64, proc_macro2::Span)> {
        match literal {
            syn::Lit::Int(value) => value
                .base10_parse::<f64>()
                .ok()
                .map(|it| (it, value.span())),
            syn::Lit::Float(value) => value
                .base10_parse::<f64>()
                .ok()
                .map(|it| (it, value.span())),
            _ => None,
        }
    }

    fn check_value(&mut self, value: f64, span: proc_macro2::Span) {
        if NumericLiteralMatcher::is_allowed(value) {
            return;
        }
        let (line, column) = SpanLocator::locate(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!("マジックナンバー `{value}` は名前付き定数へ出してください。"),
        ));
    }
}

impl<'ast> Visit<'ast> for MagicNumberVisitor {
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.const_depth += 1;
        syn::visit::visit_item_const(self, node);
        self.const_depth = self.const_depth.saturating_sub(1);
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.const_depth += 1;
        syn::visit::visit_impl_item_const(self, node);
        self.const_depth = self.const_depth.saturating_sub(1);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        self.check_literal(&node.lit);
        syn::visit::visit_expr_lit(self, node);
    }
}
