use crate::Violation;
use crate::ast::SpanLocator;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const MAX_NESTING_DEPTH: usize = 3;

pub struct NestingDepthRule;

impl NestingDepthRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = NestingDepthVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct NestingDepthVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    depth: usize,
}

impl NestingDepthVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            depth: 0,
        }
    }

    fn enter_nested(&mut self, span: proc_macro2::Span) {
        self.depth += 1;
        if self.depth <= MAX_NESTING_DEPTH {
            return;
        }
        let (line, column) = SpanLocator::locate(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!(
                "ネストが {MAX_NESTING_DEPTH} 段を超えています。現在は {} 段です。",
                self.depth
            ),
        ));
    }

    fn leave_nested(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

impl<'ast> Visit<'ast> for NestingDepthVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.enter_nested(node.if_token.span);
        syn::visit::visit_expr_if(self, node);
        self.leave_nested();
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.enter_nested(node.for_token.span);
        syn::visit::visit_expr_for_loop(self, node);
        self.leave_nested();
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.enter_nested(node.while_token.span);
        syn::visit::visit_expr_while(self, node);
        self.leave_nested();
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.enter_nested(node.match_token.span);
        syn::visit::visit_expr_match(self, node);
        self.leave_nested();
    }
}
