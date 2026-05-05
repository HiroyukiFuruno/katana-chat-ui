use crate::Violation;
use crate::ast::{MethodCallFinder, SpanLocator};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct HttpStatusRule;

impl HttpStatusRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = HttpStatusVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct HttpStatusVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    unchecked_responses: BTreeSet<String>,
}

impl HttpStatusVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            unchecked_responses: BTreeSet::new(),
        }
    }

    fn track_local(&mut self, local: &syn::Local) {
        let Some(init) = &local.init else {
            return;
        };
        if !MethodCallFinder::contains(&init.expr, "send") {
            return;
        }
        if MethodCallFinder::contains(&init.expr, "error_for_status") {
            return;
        }
        if let syn::Pat::Ident(pat_ident) = &local.pat {
            self.unchecked_responses.insert(pat_ident.ident.to_string());
        }
    }

    fn check_json_call(&mut self, node: &syn::ExprMethodCall) {
        if node.method != "json" || self.receiver_checked(&node.receiver) {
            return;
        }
        if !self.receiver_has_unchecked_send(&node.receiver) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.method.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "HTTP 応答を `.json()` する前に `.error_for_status()` を通してください。",
        ));
    }

    fn receiver_checked(&self, receiver: &syn::Expr) -> bool {
        MethodCallFinder::contains(receiver, "error_for_status")
    }

    fn receiver_has_unchecked_send(&self, receiver: &syn::Expr) -> bool {
        MethodCallFinder::contains(receiver, "send") || self.is_unchecked_response_var(receiver)
    }

    fn is_unchecked_response_var(&self, receiver: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = receiver else {
            return false;
        };
        path.path
            .get_ident()
            .is_some_and(|it| self.unchecked_responses.contains(&it.to_string()))
    }
}

impl<'ast> Visit<'ast> for HttpStatusVisitor {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        self.track_local(node);
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.check_json_call(node);
        syn::visit::visit_expr_method_call(self, node);
    }
}
