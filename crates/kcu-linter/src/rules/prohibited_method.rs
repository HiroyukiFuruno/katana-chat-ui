use crate::Violation;
use crate::ast::SpanLocator;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const PROHIBITED_METHODS: [&str; 3] = ["unwrap", "expect", "unwrap_or_default"];

pub struct ProhibitedMethodRule;

impl ProhibitedMethodRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProhibitedMethodVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProhibitedMethodVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedMethodVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_method(&mut self, node: &syn::ExprMethodCall) {
        if !PROHIBITED_METHODS.contains(&node.method.to_string().as_str()) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.method.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!(
                "`{}` は禁止です。Result/Option を明示的に扱ってください。",
                node.method
            ),
        ));
    }
}

impl<'ast> Visit<'ast> for ProhibitedMethodVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.check_method(node);
        syn::visit::visit_expr_method_call(self, node);
    }
}
