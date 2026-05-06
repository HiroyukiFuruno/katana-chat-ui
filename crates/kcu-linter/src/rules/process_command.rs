use crate::Violation;
use crate::ast::{SpanLocator, TestFileMatcher};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProcessCommandRule;

impl ProcessCommandRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        if TestFileMatcher::is_test_file(path) {
            return Vec::new();
        }
        let mut visitor = ProcessCommandVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProcessCommandVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProcessCommandVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_call(&mut self, node: &syn::ExprCall) {
        if !Self::is_command_new(node) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "`std::process::Command::new` を直接使わず、専用の境界型へ集約してください。",
        ));
    }

    fn is_command_new(node: &syn::ExprCall) -> bool {
        let syn::Expr::Path(path) = &*node.func else {
            return false;
        };
        let segments = &path.path.segments;
        let Some(last) = segments.last() else {
            return false;
        };
        let previous = segments.iter().rev().nth(1);
        last.ident == "new" && previous.is_some_and(|it| it.ident == "Command")
    }
}

impl<'ast> Visit<'ast> for ProcessCommandVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_call(node);
        syn::visit::visit_expr_call(self, node);
    }
}
